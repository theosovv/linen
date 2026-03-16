# Roadmap: Linen v1.0

## Общие параметры проекта

| Параметр | Значение |
|----------|----------|
| **Язык реализации** | Rust |
| **Hot Reload (IFRP)** | В ядре с самого начала |
| **Платформы v1.0** | Linux (primary), macOS (secondary) |
| **Стандартная библиотека** | Core + эффекты (реверб, компрессор) |
| **Целевой срок v1.0** | ~12-18 месяцев (зависит от ресурсов) |

---

## Фаза 0: Инфраструктура и подготовка (2-3 недели)

### 0.1. Настройка проекта ✅
- [x] Инициализация репозитория с `cargo`
- [x] Настройка workspace: `linen-compiler`, `linen-vm`, `linen-stdlib`, `linen-cli`
- [x] CI/CD pipeline (GitHub Actions): build, test, clippy, fmt
- [x] Настройка pre-commit hooks
- [x] Лицензия (MIT/Apache-2.0 dual)

### 0.2. Зависимости и инструменты
- [ ] Аудио backend: `cpal` (cross-platform)
- [ ] Парсер: `chumsky` или `nom` (выбор: `chumsky` — хорошие ошибки)
- [ ] Тестирование: `proptest` для property-based тестирования
- [ ] Профилирование: `pprof`, `coz`

### 0.3. Архитектурные документы
- [ ] Определение AST формата
- [ ] Спецификация байткода VM
- [ ] Формат модуля/библиотеки (`.ln` → `.lnc` compiled)
- [ ] Документирование ABI для FFI

---

## Фаза 1: Frontend — Лексер и Парсер (3-4 недели)

### 1.1. Лексер
- [ ] Определение токенов (keywords, identifiers, operators, literals)
- [ ] Поддержка комментариев (`--` line, `{- -}` block)
- [ ] Обработка whitespace и indentation (off-side rule как в Haskell)
- [ ] Escape sequences в строках
- [ ] Расширяемые числа: `440.0`, `1e-3`, `0xFF`
- [ ] Модули: `module Main where`, `import Lib`

**Тесты:**
- [ ] Unit tests для каждого токена
- [ ] Property-based tests: roundtrip tokenize → stringify
- [ ] Тесты на error recovery

### 1.2. Парсер
- [ ] Основные выражения: variables, literals, application, infix ops
- [ ] Let bindings: `let x : Type = expr in body`
- [ ] Lambda: `\x -> expr`
- [ ] Pattern matching: `match expr with | Pat -> expr`
- [ ] If-then-else: `if cond then a else b`
- [ ] Type annotations: `expr : Type`
- [ ] Type definitions: `type Maybe a = Nothing | Just a`
- [ ] Recursive functions: `let rec`

**Синтаксис FRP-специфичных конструкций:**
- [ ] Behavior literals и операции
- [ ] Event literals и комбинаторы
- [ ] Signal function syntax: `proc input -> do ...`

**Тесты:**
- [ ] Парсинг всех конструкций из спецификации
- [ ] Error messages: line/column, suggestions
- [ ] Property: pretty-printed AST roundtrips

---

## Фаза 2: Type System — Система типов (5-6 недель)

### 2.1. Основы HM-типизации
- [ ] Представление типов: `Type::Var`, `Type::Con`, `Type::Arrow`, `Type::Forall`
- [ ] Unification algorithm (Robinson)
- [ ] Algorithm W (type inference)
- [ ] Generalization и instantiation
- [ ] Kind checking для конструкторов типов

### 2.2. Расширения типовой системы
- [ ] **Type classes**: `class Num a where (+) :: a -> a -> a`
- [ ] **Instances**: `instance Num Float where ...`
- [ ] **GADTs**: `data Expr a where Lit :: Int -> Expr Int`
- [ ] **Records**: `{ field : Type }` с row polymorphism
- [ ] **Type aliases**: `type Hz = Float`

### 2.3. FRP-специфичные типы
- [ ] `Behavior a` — непрерывные сигналы
- [ ] `Event a` — дискретные события
- [ ] `SF a b` — сигнальные функции
- [ ] Rate-polymorphism: `Signal (Rate 48000) Float`
- [ ] Linear types: `Buffer ⊸ (Buffer ⊸ a) → a` (подготовка)

### 2.4. Специальные проверки
- [ ] Well-formedness типов
- [ ] Exhaustiveness pattern matching
- [ ] Отслеживание эффектов: `Audio` monad

**Тесты:**
- [ ] Positive tests: корректные программы проходят
- [ ] Negative tests: ошибки типов ловятся
- [ ] Property: inferred types are principal
- [ ] Тесты на type class resolution

---

## Фаза 3: Core — IR и Optimizations (3-4 недели)

### 3.1. Intermediate Representation
- [ ] ANF (A-Normal Form) или CPS
- [ ] Monomorphization type class dictionaries
- [ ] Lambda lifting
- [ ] Defunctionalization для first-class functions

### 3.2. Core-to-Core оптимизации
- [ ] Inlining
- [ ] Constant folding
- [ ] Dead code elimination
- [ ] Common subexpression elimination
- [ ] Loop-invariant code motion (для сигналов)

### 3.3. FRP-специфичные оптимизации
- [ ] Signal fusion: `map f >>> map g` → `map (f . g)`
- [ ] Event coalescing
- [ ] Static graph analysis

**Тесты:**
- [ ] Property: optimizations preserve semantics
- [ ] Benchmark suite для оптимизаций

---

## Фаза 4: VM — Виртуальная машина (6-7 недель)

### 4.1. Bytecode Design
- [ ] Определение opcodes (stack-based)
- [ ] Формат константного пула
- [ ] Function prologue/epilogue
- [ ] Exception handling (для audio errors)

**Категории инструкций:**
```
Stack:      PUSH, POP, DUP, SWAP, ROT
Arithmetic: FADD, FMUL, FSUB, FDIV, FNEG
Logic:      FEQ, FLT, FGT, AND, OR, NOT
Memory:     LOAD, STORE, LOAD_CONST, ALLOCA
Control:    JUMP, JUMP_IF, CALL, RET, TAILCALL
Audio:      OSC_SIN, OSC_SAW, ENV_ADSR, BIQUAD
Graph:      NODE_CREATE, CONNECT, DISCONNECT
Events:     EVENT_MERGE, EVENT_MAP, EVENT_FILTER
```

### 4.2. VM Implementation
- [ ] Stack machine core
- [ ] Call stack и frame management
- [ ] Garbage collection (ref counting + cycle detector, не в audio thread!)
- [ ] FFI interface для C библиотек

### 4.3. Audio Thread Architecture
- [ ] **Двухпоточная модель:**
  - Audio thread: real-time, lock-free, никаких аллокаций
  - Control thread: всё остальное (GC, JIT, I/O)
- [ ] Lock-free ring buffer для общения
- [ ] Lock-free command queue (Michael-Scott)

### 4.4. Memory Management
- [ ] Region-based allocation для audio frames
- [ ] Object pools для часто используемых структур
- [ ] Linear types: runtime checks или compile-time (подготовка)
- [ ] Double/triple buffering для аудио

**Тесты:**
- [ ] VM unit tests (each opcode)
- [ ] Stress tests (long-running)
- [ ] Memory leak tests (valgrind/miri)
- [ ] Real-time safety tests (no syscalls in audio thread)

---

## Фаза 5: Audio Backend — Низкоуровневое аудио (4-5 недель)

### 5.1. Audio Abstraction Layer
- [ ] Trait `AudioBackend`:
  ```rust
  trait AudioBackend {
    fn start(&mut self, callback: AudioCallback) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn sample_rate(&self) -> u32;
    fn buffer_size(&self) -> usize;
  }
  ```

### 5.2. Backend Implementations
- [ ] **CPAL** (cross-platform fallback)
  - Linux: ALSA через CPAL
  - macOS: CoreAudio через CPAL
- [ ] **JACK** (Linux, опционально для v1.0)
  - Низкая latency
  - Межприложенная маршрутизация

### 5.3. Real-time Guarantees
- [ ] `SCHED_FIFO` priority для audio thread
- [ ] `mlockall()` для предотвращения swapping
- [ ] CPU affinity (dedicated cores)
- [ ] No allocation in audio callback
- [ ] Lock-free всё

### 5.4. Audio Primitives (начальная реализация)
- [ ] Phasor (фазовый аккумулятор)
- [ ] Sine oscillator (table-based)
- [ ] Audio I/O: `audioIn` / `audioOut`
- [ ] Gain / pan

**Тесты:**
- [ ] Тест tone generation (sin 440Hz)
- [ ] Latency measurement
- [ ] Glitch detection (zero crossings, DC offset)
- [ ] Backend switching tests

---

## Фаза 6: IFRP — Инкрементальное FRP (8-10 недель)

### 6.1. Term Labeling
- [ ] Присвоение уникальных ID каждому stateful узлу
- [ ] AST annotation pass
- [ ] Source location → label mapping
- [ ] Explicit labels: `#[label("lfo")]`

### 6.2. State Representation
- [ ] `StateMap: Label -> Box<dyn AnyState>`
- [ ] Type-safe state serialization
- [ ] State versioning

### 6.3. Graph Diff Algorithm
- [ ] AST comparison: structural matching
- [ ] Label mapping: old -> new
- [ ] Detection: preserved / modified / added / removed
- [ ] Topological order preservation

### 6.4. State Migration
- [ ] Copy state for preserved nodes
- [ ] Initialize new nodes with defaults
- [ ] Parameter interpolation (crossfade ~10ms)
- [ ] Cleanup removed nodes

### 6.5. Hot Reload Pipeline
- [ ] File watcher (inotify/FSEvents)
- [ ] Incremental recompilation
- [ ] Atomic state swap
- [ ] Error recovery (rollback on failure)

### 6.6. Audio State Preservation
- [ ] Oscillator phases
- [ ] Envelope positions
- [ ] Delay line contents
- [ ] Filter states (z⁻¹, z⁻²)
- [ ] Sequencer positions

**Тесты:**
- [ ] Property: state is preserved across reloads
- [ ] Stress tests: rapid reloads
- [ ] Error cases: invalid code → graceful degradation
- [ ] Audio continuity tests (no clicks/pops)

---

## Фаза 7: JIT-компиляция (6-8 недель)

### 7.1. JIT Infrastructure
- [ ] Cranelift integration (компиляция в фоне)
- [ ] Tiered compilation:
  - Tier 0: Interpreter + profiling
  - Tier 1: Cranelift quick compile
  - Tier 2: Cranelift optimize (второй проход)

### 7.2. Profiling
- [ ] Basic block counters
- [ ] Execution time tracking
- [ ] Hotspot detection

### 7.3. Compilation Triggers
- [ ] Call count > threshold
- [ ] Execution time > 10% frame budget
- [ ] Explicit annotation: `#[jit]`
- [ ] Stability: code unchanged > 5 seconds

### 7.4. Optimizations
- [ ] Loop unrolling для аудио блоков
- [ ] SIMD vectorization (f32x4, f32x8)
- [ ] Function inlining
- [ ] Constant propagation
- [ ] Dead store elimination

### 7.5. Code Replacement
- [ ] Atomic pointer swap
- [ ] Version counters
- [ ] Safe deallocation старого кода

**Тесты:**
- [ ] Correctness: JIT == interpreter
- [ ] Performance benchmarks
- [ ] Compilation time limits
- [ ] Graceful degradation under load

---

## Фаза 8: Core Standard Library — Ядро стандартной библиотеки (6-7 недель)

### 8.1. Signal Primitives
```
sinOsc   :: Frequency -> Behavior Float
sawOsc   :: Frequency -> Behavior Float  
squareOsc:: Frequency -> Behavior Float
triOsc   :: Frequency -> Behavior Float
```
- [ ] Table-based oscillators
- [ ] Linear interpolation
- [ ] BLEP для saw/square (bandlimited)
- [ ] Phase preservation

### 8.2. Envelopes
```
adsr :: ADSRParams -> Event () -> Event () -> Behavior Float
```
- [ ] State machine: Idle → Attack → Decay → Sustain → Release
- [ ] Exponential curves
- [ ] Velocity scaling
- [ ] Parameter modulation

### 8.3. Filters (Biquad)
```
lpf :: Frequency -> Q -> Behavior Float -> Behavior Float
hpf :: Frequency -> Q -> Behavior Float -> Behavior Float  
bpf :: Frequency -> Q -> Behavior Float -> Behavior Float
notch:: Frequency -> Q -> Behavior Float -> Behavior Float
```
- [ ] Direct Form II
- [ ] State preservation
- [ ] Frequency response accuracy
- [ ] Coefficient smoothing

### 8.4. FRP Combinators
```
-- Behaviors
pure    :: a -> Behavior a
liftA2  :: (a -> b -> c) -> Behavior a -> Behavior b -> Behavior c
integral:: Behavior Float -> Behavior Float
sample  :: Behavior a -> Event b -> Event a

-- Events
never   :: Event a
merge   :: Event a -> Event a -> Event a
filterE :: (a -> Bool) -> Event a -> Event a
mapE    :: (a -> b) -> Event a -> Event b
snapshot:: Event a -> Behavior b -> Event (a, b)
hold    :: a -> Event a -> Behavior a
accum   :: a -> Event (a -> a) -> Behavior a

-- Signal Functions
arr     :: (a -> b) -> SF a b
(>>>)   :: SF a b -> SF b c -> SF a c
(&&&)   :: SF a b -> SF a c -> SF a (b, c)
( ***)  :: SF a b -> SF c d -> SF (a, c) (b, d)
loop    :: SF (a, c) (b, c) -> SF a b
```

### 8.5. Time Utilities
```
setBPM      :: BPM -> Audio ()
tapTempo    :: Event () -> Behavior BPM
transport   :: Behavior TransportState
isPlaying   :: Behavior Bool
position    :: Behavior MusicalTime
quantize    :: Duration -> Event a -> Event a
```

### 8.6. I/O
```
-- Audio files
readWav  :: FilePath -> Audio (Behavior [Float])
writeWav :: FilePath -> Behavior [Float] -> Audio ()

-- MIDI
midiIn   :: Audio (Event MidiMessage)
midiNote :: Channel -> Event (Note, Velocity)
midiCC   :: Channel -> CC -> Behavior Float

-- Audio interface
audioOut :: Behavior [Float] -> Audio ()
audioIn  :: Audio (Behavior [Float])
```

**Тесты:**
- [ ] Unit tests для каждого примитива
- [ ] Property-based: filter frequency response
- [ ] Golden tests: compare с reference implementation
- [ ] Audio file regression tests

---

## Фаза 9: Effects — Эффекты обработки (5-6 недель)

### 9.1. Delay Effects
```
delay    :: Time -> Feedback -> Mix -> Behavior Float -> Behavior Float
multiTap :: [(Time, Gain)] -> Behavior Float -> Behavior Float
pingPong :: Time -> Feedback -> Behavior Float -> Behavior [Float]
```
- [ ] Кольцевые буферы
- [ ] Интерполяция задержки
- [ ] State preservation

### 9.2. Algorithmic Reverb
```
freeVerb :: RoomSize -> Damping -> Width -> DryWet -> Behavior Float -> Behavior [Float]
```
- [ ] All-pass filters
- [ ] Comb filters
- [ ] Модульная архитектура (Schroeder, Moorer)
- [ ] Stereo width

### 9.3. Dynamics
```
compressor :: Threshold -> Ratio -> Attack -> Release -> Behavior Float -> Behavior Float
limiter  :: Ceiling -> Lookahead -> Release -> Behavior Float -> Behavior Float
gate     :: Threshold -> Range -> Attack -> Hold -> Release -> Behavior Float -> Behavior Float
```
- [ ] Peak и RMS detection
- [ ] Soft knee
- [ ] Sidechain input
- [ ] Lookahead buffer

### 9.4. Modulation Effects
```
chorus   :: Rate -> Depth -> Mix -> Behavior Float -> Behavior Float
flanger  :: Rate -> Depth -> Feedback -> Mix -> Behavior Float -> Behavior Float
phaser   :: Rate -> Depth -> Stages -> Mix -> Behavior Float -> Behavior Float
```
- [ ] LFO-driven delay/phase
- [ ] Through-zero flanger

### 9.5. Distortion
```
softClip :: Amount -> Behavior Float -> Behavior Float
hardClip :: Threshold -> Behavior Float -> Behavior Float
tanhDist :: Drive -> Behavior Float -> Behavior Float
bitCrush :: Bits -> Behavior Float -> Behavior Float
```

**Тесты:**
- [ ] Impulse response tests
- [ ] Frequency response analysis
- [ ] THD+N measurements
- [ ] State preservation при hot reload

---

## Фаза 10: CLI и Developer Tools (3-4 недели)

### 10.1. CLI Interface
```bash
linen --help              # Помощь
linen run file.ln         # Выполнение
linen compile file.ln -o out.lnc  # Компиляция
linen repl                # Интерактивная сессия
linen check file.ln       # Проверка типов
linen doc                 # Генерация документации
```

### 10.2. REPL
- [ ] Expression evaluation
- [ ] :type команда
- [ ] :reload для hot reload
- [ ] История (readline)
- [ ] Tab completion
- [ ] Pretty printing values

### 10.3. Diagnostics
- [ ] Error messages с source locations
- [ ] Suggestions ("Did you mean...?")
- [ ] Contextual help
- [ ] Warning levels

### 10.4. Package Manager (базовый)
- [ ] `linen.toml` формат
- [ ] `dependencies` section
- [ ] `linen install`
- [ ] Version resolution

### 10.5. Debugger / Profiler
- [ ] Signal graph visualization
- [ ] Real-time scope (осциллограф)
- [ ] Spectrum analyzer
- [ ] CPU usage per node
- [ ] Memory profiler

**Тесты:**
- [ ] CLI integration tests
- [ ] REPL session tests
- [ ] Error message quality tests

---

## Фаза 11: macOS Support (3-4 недели)

### 11.1. CoreAudio Backend
- [ ] CPAL уже поддерживает CoreAudio
- [ ] Оптимизации под macOS
- [ ] Bundle packaging

### 11.2. Platform-specific Code
- [ ] Memory locking: `mlock` вместо `mlockall`
- [ ] Thread priorities
- [ ] File watching: FSEvents

### 11.3. CI/CD
- [ ] GitHub Actions runner для macOS
- [ ] Universal binary (Intel + Apple Silicon)
- [ ] Code signing (опционально для v1.0)

**Тесты:**
- [ ] Audio hardware compatibility
- [ ] Performance benchmarks

---

## Фаза 12: Documentation и Release (3-4 недели)

### 12.1. User Documentation
- [ ] Language tutorial ("Learn Linen in Y minutes")
- [ ] FRP concepts explained
- [ ] Standard library reference
- [ ] Cookbook: common patterns
- [ ] Video tutorials (опционально)

### 12.2. Developer Documentation
- [ ] Contributing guide
- [ ] Architecture overview
- [ ] Hacking on the compiler
- [ ] FFI guide
- [ ] Performance tuning

### 12.3. Examples
- [ ] `examples/hello_sine.ln`
- [ ] `examples/subtractive_synth.ln`
- [ ] `examples/fm_bell.ln`
- [ ] `examples/drum_machine.ln`
- [ ] `examples/granular_cloud.ln`
- [ ] `examples/live_coding_session.ln`

### 12.4. Website
- [ ] Homepage
- [ ] Documentation site (mdBook)
- [ ] Playground (WASM компиляция)

### 12.5. Release
- [ ] Version tagging
- [ ] Release notes
- [ ] Binary builds для Linux/macOS
- [ ] Cargo publish
- [ ] Announcement

---

## Итоговая воронка задач

| Фаза | Длительность | Основные результаты | Критерий готовности |
|------|--------------|---------------------|---------------------|
| 0 | 2-3 недели | Инфраструктура | CI зелёный, hello world компилируется |
| 1 | 3-4 недели | Парсер | Весь синтаксис из спеки парсится |
| 2 | 5-6 недель | Type checker | Программы типизируются корректно |
| 3 | 3-4 недели | Optimized IR | Core проходит оптимизации |
| 4 | 6-7 недель | VM | Байткод выполняется |
| 5 | 4-5 недель | Audio backend | Звук выводится |
| 6 | 8-10 недель | IFRP | Hot reload работает |
| 7 | 6-8 недель | JIT | Ускорение по сравнению с интерпретатором |
| 8 | 6-7 недель | Core stdlib | Осцилляторы, фильтры, ADSR |
| 9 | 5-6 недель | Effects | Реверб, компрессор, etc. |
| 10 | 3-4 недели | CLI | Полный user experience |
| 11 | 3-4 недели | macOS | Работает на macOS |
| 12 | 3-4 недели | Release | Документация, examples |

**Общая длительность:** ~12-18 месяцев (1 человек full-time)
**Параллелизация:** Фазы 7 (JIT) и 8-9 (stdlib) могут идти параллельно

---

## MVP Definition (v0.1 — 3 месяца)

Для быстрого получения обратной связи:

- Парсер + Type checker (без расширений)
- Интерпретатор (без VM/JIT)
- CPAL backend (без JACK)
- Только sine oscillator, gain, audioOut
- Базовый IFRP (без интерполяции)
- CLI run, без REPL

**MVP критерий:** Можно написать `sinOsc 440 |> gain 0.5 |> audioOut`, сохранить файл, услышать звук, изменить частоту, пересохранить — звук обновился без щелчка.

---

## Риски и Mitigation

| Риск | Вероятность | Влияние | Mitigation |
|------|-------------|---------|------------|
| IFRP сложнее ожидаемого | Средняя | Высокое | MVP без него, добавить позже |
| JIT не даёт прироста | Низкая | Среднее | Сфокусироваться на VM оптимизациях |
| Real-time гарантии на macOS | Средняя | Среднее | Использовать CPAL, fallback к buffering |
| Rust learning curve | Низкая | Низкое | Начать с простых компонентов |

---

## Метрики успеха v1.0

- [ ] Latency < 10ms на Linux с JACK
- [ ] CPU usage < 10% для 10 голосов полифонии
- [ ] Hot reload < 100ms
- [ ] Стабильность: 24 часа работы без сбоев
- [ ] Тестовое покрытие > 80%
- [ ] Документация на все публичные API
