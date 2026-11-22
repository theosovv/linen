#![allow(dead_code)]

use linen::sema::{
    BindingId, BuiltinId, Type, TypedBinding, TypedExpr, TypedExprKind, TypedProgram,
};
use linen::syntax::{BinaryOp, UnaryOp};

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceErrorKind {
    NonFiniteFrequency,
    NonFinitePosition,
    NonFiniteOutput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceError {
    pub kind: ReferenceErrorKind,
    pub sample_index: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferenceFrame {
    Mono(f32),
    Stereo([f32; 2]),
}

#[derive(Clone, Copy, Debug)]
struct ReferenceNoiseState {
    seed: u64,
    state: u64,
    initialized: bool,
}

pub struct ReferenceInterpreter {
    bindings: Vec<ReferenceBinding>,
    output: BindingId,
    phases: Vec<f64>,
    noise: Vec<ReferenceNoiseState>,
    slot_count: usize,
}

struct ReferenceBinding {
    id: BindingId,
    value: ReferenceExpr,
}

struct ReferenceExpr {
    kind: ReferenceExprKind,
    ty: Type,
}

#[derive(Clone, Copy)]
enum ReferenceOscillator {
    Phasor,
    Saw,
    Square,
    Triangle,
}

enum ReferenceExprKind {
    Number(f64),
    Identifier(BindingId),
    Sine {
        frequency: Box<ReferenceExpr>,
        state: usize,
    },
    Oscillator {
        oscillator: ReferenceOscillator,
        frequency: Box<ReferenceExpr>,
        state: usize,
    },
    Gain {
        input: Box<ReferenceExpr>,
        amount: Box<ReferenceExpr>,
    },
    Mix {
        a: Box<ReferenceExpr>,
        b: Box<ReferenceExpr>,
        balance: Box<ReferenceExpr>,
    },
    Stereo {
        left: Box<ReferenceExpr>,
        right: Box<ReferenceExpr>,
    },
    Pan {
        input: Box<ReferenceExpr>,
        position: Box<ReferenceExpr>,
    },
    Noise {
        seed: Box<ReferenceExpr>,
        state: usize,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<ReferenceExpr>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<ReferenceExpr>,
        right: Box<ReferenceExpr>,
    },
    Group(Box<ReferenceExpr>),
    Block {
        bindings: Vec<ReferenceBinding>,
        result: Box<ReferenceExpr>,
    },
    Lift(Box<ReferenceExpr>),
}

#[derive(Clone, Copy)]
enum ReferenceValue {
    Float(f64),
    Signal(f32),
    Stereo([f32; 2]),
}

struct ReferenceBuilder {
    phases: Vec<f64>,
    noise: Vec<ReferenceNoiseState>,
    slot_count: usize,
}

impl ReferenceBuilder {
    fn register(&mut self, id: BindingId) {
        self.slot_count = self.slot_count.max(id.index() + 1);
    }

    fn phase(&mut self) -> usize {
        let state = self.phases.len();
        self.phases.push(0.0);
        state
    }

    fn noise(&mut self) -> usize {
        let state = self.noise.len();
        self.noise.push(ReferenceNoiseState {
            seed: 0,
            state: 0,
            initialized: false,
        });
        state
    }
}

impl ReferenceInterpreter {
    pub fn new(program: &TypedProgram) -> Self {
        let mut builder = ReferenceBuilder {
            phases: Vec::new(),
            noise: Vec::new(),
            slot_count: 0,
        };
        let bindings = program
            .bindings()
            .into_iter()
            .map(|binding| {
                builder.register(binding.id);
                ReferenceBinding {
                    id: binding.id,
                    value: lower_expr(&binding.value, program, &mut builder),
                }
            })
            .collect();
        builder.register(program.output);
        Self {
            bindings,
            output: program.output,
            phases: builder.phases,
            noise: builder.noise,
            slot_count: builder.slot_count,
        }
    }

    pub fn reset(&mut self) {
        self.phases.fill(0.0);
        for state in &mut self.noise {
            state.initialized = false;
            state.state = state.seed;
        }
    }

    pub fn next_sample(
        &mut self,
        sample_index: u64,
        sample_rate: u32,
    ) -> Result<f32, ReferenceError> {
        match self.next_frame(sample_index, sample_rate)? {
            ReferenceFrame::Mono(sample) => Ok(sample),
            ReferenceFrame::Stereo(_) => panic!("mono next_sample cannot return Stereo"),
        }
    }

    pub fn next_frame(
        &mut self,
        sample_index: u64,
        sample_rate: u32,
    ) -> Result<ReferenceFrame, ReferenceError> {
        let mut values = vec![None; self.slot_count];

        for index in 0..self.bindings.len() {
            let id = self.bindings[index].id;
            let value = eval(
                &self.bindings[index].value,
                &mut values,
                &mut self.phases,
                &mut self.noise,
                sample_index,
                sample_rate,
            )?;
            values[id.index()] = Some(value);
        }

        let output = values[self.output.index()]
            .expect("the output binding must be evaluated before output");
        match output {
            ReferenceValue::Signal(sample) => {
                check_finite(sample, sample_index)?;
                Ok(ReferenceFrame::Mono(sample))
            }
            ReferenceValue::Stereo(channels) => {
                for sample in channels {
                    check_finite(sample, sample_index)?;
                }
                Ok(ReferenceFrame::Stereo(channels))
            }
            ReferenceValue::Float(_) => {
                panic!("typed output must be audio after its explicit lift")
            }
        }
    }
}

fn check_finite(sample: f32, sample_index: u64) -> Result<(), ReferenceError> {
    if sample.is_finite() {
        return Ok(());
    }
    Err(ReferenceError {
        kind: ReferenceErrorKind::NonFiniteOutput,
        sample_index,
    })
}

fn lower_bindings(
    bindings: &[TypedBinding],
    program: &TypedProgram,
    builder: &mut ReferenceBuilder,
) -> Vec<ReferenceBinding> {
    bindings
        .iter()
        .map(|binding| {
            builder.register(binding.id);
            ReferenceBinding {
                id: binding.id,
                value: lower_expr(&binding.value, program, builder),
            }
        })
        .collect()
}

fn lower_expr(
    expr: &TypedExpr,
    program: &TypedProgram,
    builder: &mut ReferenceBuilder,
) -> ReferenceExpr {
    let kind = match &expr.kind {
        TypedExprKind::Number(value) => ReferenceExprKind::Number(*value),
        TypedExprKind::Identifier { id, .. } => {
            builder.register(*id);
            ReferenceExprKind::Identifier(*id)
        }
        TypedExprKind::BuiltinCall {
            builtin, arguments, ..
        } => match *builtin {
            BuiltinId::SINE => {
                assert_eq!(arguments.len(), 1, "sine must have one checked argument");
                ReferenceExprKind::Sine {
                    frequency: Box::new(lower_expr(&arguments[0], program, builder)),
                    state: builder.phase(),
                }
            }
            BuiltinId::PHASOR | BuiltinId::SAW | BuiltinId::SQUARE | BuiltinId::TRIANGLE => {
                assert_eq!(
                    arguments.len(),
                    1,
                    "oscillator must have one checked argument"
                );
                let oscillator = match *builtin {
                    BuiltinId::PHASOR => ReferenceOscillator::Phasor,
                    BuiltinId::SAW => ReferenceOscillator::Saw,
                    BuiltinId::SQUARE => ReferenceOscillator::Square,
                    BuiltinId::TRIANGLE => ReferenceOscillator::Triangle,
                    _ => panic!("matched oscillator builtin"),
                };
                ReferenceExprKind::Oscillator {
                    oscillator,
                    frequency: Box::new(lower_expr(&arguments[0], program, builder)),
                    state: builder.phase(),
                }
            }
            BuiltinId::GAIN => {
                assert_eq!(arguments.len(), 2, "gain must have two checked arguments");
                ReferenceExprKind::Gain {
                    input: Box::new(lower_expr(&arguments[0], program, builder)),
                    amount: Box::new(lower_expr(&arguments[1], program, builder)),
                }
            }
            BuiltinId::MIX => {
                assert_eq!(arguments.len(), 3, "mix must have three checked arguments");
                ReferenceExprKind::Mix {
                    a: Box::new(lower_expr(&arguments[0], program, builder)),
                    b: Box::new(lower_expr(&arguments[1], program, builder)),
                    balance: Box::new(lower_expr(&arguments[2], program, builder)),
                }
            }
            BuiltinId::STEREO => {
                assert_eq!(arguments.len(), 2, "stereo must have two checked arguments");
                ReferenceExprKind::Stereo {
                    left: Box::new(lower_expr(&arguments[0], program, builder)),
                    right: Box::new(lower_expr(&arguments[1], program, builder)),
                }
            }
            BuiltinId::PAN => {
                assert_eq!(arguments.len(), 2, "pan must have two checked arguments");
                ReferenceExprKind::Pan {
                    input: Box::new(lower_expr(&arguments[0], program, builder)),
                    position: Box::new(lower_expr(&arguments[1], program, builder)),
                }
            }
            BuiltinId::NOISE => {
                assert_eq!(arguments.len(), 1, "noise must have one checked argument");
                ReferenceExprKind::Noise {
                    seed: Box::new(lower_expr(&arguments[0], program, builder)),
                    state: builder.noise(),
                }
            }
            _ => panic!("unsupported reference builtin"),
        },
        TypedExprKind::FunctionCall {
            function,
            arguments,
            ..
        } => {
            let definition = program
                .functions()
                .find(|candidate| candidate.id == *function)
                .expect("a resolved function must have a typed definition");
            assert_eq!(
                arguments.len(),
                definition.parameters.len(),
                "function arity must be checked before reference construction"
            );
            let mut bindings = Vec::with_capacity(arguments.len());
            for (argument, parameter) in arguments.iter().zip(&definition.parameters) {
                builder.register(parameter.id);
                let value = lower_expr(argument, program, builder);
                bindings.push(ReferenceBinding {
                    id: parameter.id,
                    value,
                });
            }
            let result = Box::new(lower_expr(&definition.body, program, builder));
            ReferenceExprKind::Block { bindings, result }
        }
        TypedExprKind::Unary { operator, operand } => ReferenceExprKind::Unary {
            operator: *operator,
            operand: Box::new(lower_expr(operand, program, builder)),
        },
        TypedExprKind::Binary {
            operator,
            left,
            right,
        } => ReferenceExprKind::Binary {
            operator: *operator,
            left: Box::new(lower_expr(left, program, builder)),
            right: Box::new(lower_expr(right, program, builder)),
        },
        TypedExprKind::Group(inner) => {
            ReferenceExprKind::Group(Box::new(lower_expr(inner, program, builder)))
        }
        TypedExprKind::Block { bindings, result } => ReferenceExprKind::Block {
            bindings: lower_bindings(bindings, program, builder),
            result: Box::new(lower_expr(result, program, builder)),
        },
        TypedExprKind::LiftFloatToSig(inner) => {
            ReferenceExprKind::Lift(Box::new(lower_expr(inner, program, builder)))
        }
    };
    ReferenceExpr { kind, ty: expr.ty }
}

#[allow(clippy::too_many_arguments)]
fn eval(
    expr: &ReferenceExpr,
    values: &mut [Option<ReferenceValue>],
    phases: &mut [f64],
    noise: &mut [ReferenceNoiseState],
    sample_index: u64,
    sample_rate: u32,
) -> Result<ReferenceValue, ReferenceError> {
    match &expr.kind {
        ReferenceExprKind::Number(value) => Ok(ReferenceValue::Float(*value)),
        ReferenceExprKind::Identifier(id) => {
            Ok(values[id.index()].expect("resolved binding must be evaluated before use"))
        }
        ReferenceExprKind::Sine { frequency, state } => {
            let frequency = signal_value(eval(
                frequency,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            let phase = advance_phase(phases, *state, frequency, sample_rate, sample_index)?;
            Ok(ReferenceValue::Signal(
                (std::f64::consts::TAU * phase).sin() as f32,
            ))
        }
        ReferenceExprKind::Oscillator {
            oscillator,
            frequency,
            state,
        } => {
            let frequency = signal_value(eval(
                frequency,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            let phase = advance_phase(phases, *state, frequency, sample_rate, sample_index)?;
            let sample = match oscillator {
                ReferenceOscillator::Phasor => phase,
                ReferenceOscillator::Saw => 2.0 * phase - 1.0,
                ReferenceOscillator::Square if phase < 0.5 => 1.0,
                ReferenceOscillator::Square => -1.0,
                ReferenceOscillator::Triangle if phase < 0.5 => 4.0 * phase - 1.0,
                ReferenceOscillator::Triangle => 3.0 - 4.0 * phase,
            };
            Ok(ReferenceValue::Signal(sample as f32))
        }
        ReferenceExprKind::Gain { input, amount } => {
            let input = signal_value(eval(
                input,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            let amount = signal_value(eval(
                amount,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            Ok(ReferenceValue::Signal(input * amount))
        }
        ReferenceExprKind::Mix { a, b, balance } => {
            let a = signal_value(eval(a, values, phases, noise, sample_index, sample_rate)?);
            let b = signal_value(eval(b, values, phases, noise, sample_index, sample_rate)?);
            let balance = signal_value(eval(
                balance,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            Ok(ReferenceValue::Signal(a * (1.0 - balance) + b * balance))
        }
        ReferenceExprKind::Stereo { left, right } => {
            let left = signal_value(eval(
                left,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            let right = signal_value(eval(
                right,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            Ok(ReferenceValue::Stereo([left, right]))
        }
        ReferenceExprKind::Pan { input, position } => {
            let input = signal_value(eval(
                input,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            let position = signal_value(eval(
                position,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            if !position.is_finite() {
                return Err(ReferenceError {
                    kind: ReferenceErrorKind::NonFinitePosition,
                    sample_index,
                });
            }
            let position = f64::from(position.clamp(-1.0, 1.0));
            let angle = (position + 1.0) * std::f64::consts::FRAC_PI_4;
            let left = input * angle.cos() as f32;
            let right = input * angle.sin() as f32;
            Ok(ReferenceValue::Stereo([left, right]))
        }
        ReferenceExprKind::Noise { seed, state } => {
            let seed_value = float_value(eval(
                seed,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            assert_valid_seed(seed_value);
            let state = &mut noise[*state];
            if !state.initialized {
                state.seed = seed_value as u64;
                state.state = state.seed;
                state.initialized = true;
            }
            state.state = state.state.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = state.state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^= z >> 31;
            let sample = ((z >> 40) as u32) as f32 / 8_388_608.0 - 1.0;
            Ok(ReferenceValue::Signal(sample))
        }
        ReferenceExprKind::Unary { operator, operand } => {
            let operand = eval(operand, values, phases, noise, sample_index, sample_rate)?;
            match (operator, operand) {
                (UnaryOp::Negate, ReferenceValue::Float(value)) => {
                    Ok(ReferenceValue::Float(-value))
                }
                (UnaryOp::Negate, ReferenceValue::Signal(value)) => {
                    Ok(ReferenceValue::Signal(-value))
                }
                (UnaryOp::Negate, ReferenceValue::Stereo(_)) => {
                    panic!("Stereo unary arithmetic is rejected by type checking")
                }
            }
        }
        ReferenceExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = eval(left, values, phases, noise, sample_index, sample_rate)?;
            let right = eval(right, values, phases, noise, sample_index, sample_rate)?;
            match expr.ty {
                Type::Float => {
                    let left = float_value(left);
                    let right = float_value(right);
                    let value = match operator {
                        BinaryOp::Add => left + right,
                        BinaryOp::Subtract => left - right,
                        BinaryOp::Multiply => left * right,
                        BinaryOp::Divide => left / right,
                    };
                    Ok(ReferenceValue::Float(value))
                }
                Type::Sig => {
                    let left = signal_value(left);
                    let right = signal_value(right);
                    let value = match operator {
                        BinaryOp::Add => left + right,
                        BinaryOp::Subtract => left - right,
                        BinaryOp::Multiply => left * right,
                        BinaryOp::Divide => left / right,
                    };
                    Ok(ReferenceValue::Signal(value))
                }
                Type::Stereo => panic!("Stereo binary arithmetic is rejected by type checking"),
            }
        }
        ReferenceExprKind::Group(inner) => {
            eval(inner, values, phases, noise, sample_index, sample_rate)
        }
        ReferenceExprKind::Block { bindings, result } => {
            let mut saved = Vec::with_capacity(bindings.len());
            for binding in bindings {
                let value = eval(
                    &binding.value,
                    values,
                    phases,
                    noise,
                    sample_index,
                    sample_rate,
                )?;
                saved.push((binding.id, values[binding.id.index()]));
                values[binding.id.index()] = Some(value);
            }
            let result = eval(result, values, phases, noise, sample_index, sample_rate);
            for (id, value) in saved.into_iter().rev() {
                values[id.index()] = value;
            }
            result
        }
        ReferenceExprKind::Lift(inner) => {
            let value = float_value(eval(
                inner,
                values,
                phases,
                noise,
                sample_index,
                sample_rate,
            )?);
            Ok(ReferenceValue::Signal(value as f32))
        }
    }
}

fn advance_phase(
    phases: &mut [f64],
    state: usize,
    frequency: f32,
    sample_rate: u32,
    sample_index: u64,
) -> Result<f64, ReferenceError> {
    if !frequency.is_finite() {
        return Err(ReferenceError {
            kind: ReferenceErrorKind::NonFiniteFrequency,
            sample_index,
        });
    }
    let phase = phases[state];
    phases[state] = (phase + f64::from(frequency) / f64::from(sample_rate)).rem_euclid(1.0);
    Ok(phase)
}

fn assert_valid_seed(value: f64) {
    assert!(
        value.is_finite()
            && (0.0..=9_007_199_254_740_991.0).contains(&value)
            && value.fract() == 0.0,
        "reference programs must contain a checked noise seed"
    );
}

fn float_value(value: ReferenceValue) -> f64 {
    match value {
        ReferenceValue::Float(value) => value,
        ReferenceValue::Signal(_) | ReferenceValue::Stereo(_) => {
            panic!("a Float expression cannot contain a runtime signal")
        }
    }
}

fn signal_value(value: ReferenceValue) -> f32 {
    match value {
        ReferenceValue::Signal(value) => value,
        ReferenceValue::Float(_) => panic!("mixed signals must use an explicit lift"),
        ReferenceValue::Stereo(_) => panic!("a mono operation cannot consume Stereo"),
    }
}
