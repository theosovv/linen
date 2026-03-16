# Roadmap: Linen v1.0

## Общие параметры проекта

| Параметр | Значение |
|----------|----------|
| **Язык реализации** | Rust |
| **Hot Reload (IFRP)** | В ядре с самого начала |
| **Платформы v1.0** | Linux (primary), macOS (secondary) |
| **Стандартная библиотека** | Core + эффекты (реверб, компрессор) |
| **Целевой срок v1.0** | ~12-18 месяцев |
| **Архитектура** | [architecture.md](architecture.md) (implementation) |
| **Язык** | [language-spec.md](language-spec.md) (user-facing) |

---

## Фаза 0: Инфраструктура и подготовка ✅

### 0.1. Настройка проекта ✅
- [x] Инициализация репозитория с `cargo`
- [x] Настройка workspace: `linen-compiler`, `linen-vm`, `linen-stdlib`, `linen-cli`
- [x] CI/CD pipeline (GitHub Actions): build, test, clippy, fmt
- [x] Настройка pre-commit hooks
- [x] Лицензия (MIT/Apache-2.0 dual)

### 0.2. Зависимости и инструменты ✅
- [x] Парсер: `chumsky` (в `linen-compiler`)
- [x] Тестирование: `proptest` (workspace-level)
- [ ] Аудио backend: `cpal` (добавится в фазе 5)
- [ ] Профилирование: `pprof`, `coz` (добавятся при оптимизации)

### 0.3. Архитектурные документы ✅
- [x] Compiler Pipeline ([architecture.md](architecture.md) §1)
- [x] AST Format с макросами ([architecture.md](architecture.md) §2)
- [x] Bytecode с Audio/Control контекстами и Linear Types ([architecture.md](architecture.md) §3)
- [x] Module Format с Hot Reload поддержкой ([architecture.md](architecture.md) §4)
- [x] FFI ABI для built-in библиотек ([architecture.md](architecture.md) §5)

### 0.4. Документация для пользователей ✅
- [x] Language Specification ([language-spec.md](language-spec.md))
- [x] Syntax Quick Reference ([syntax-summary.md](syntax-summary.md))
- [x] Cross-Reference ([cross-reference.md](cross-reference.md))

---

## Фаза 1: Frontend — Лексер и Парсер (3-4 недели)

**См. [architecture.md](architecture.md) §2 для AST формата, [language-spec.md](language-spec.md) для синтаксиса.**

### 1.1. Лексер
- [ ] Определение токенов ([architecture.md](architecture.md) §2.1, [language-spec.md](language-spec.md) §2, §4.9)
  - Keywords: `let`, `let rec`, `type`, `match`, `if`, `then`, `else`, `in`
  - FRP keywords: `behavior`, `event`, `proc`, `returnA`
  - Macro keywords: `quote`, с `!` суффиксом
  - Operators: `->`, `=>`, `::`, `|`, `+`, `-`, `*`, `/`, `>>>`, `&&&`
  - Delimiters: `()`, `{}`, `[]`, `<-` (proc bind)
- [ ] Поддержка комментариев (`--` line, `{- -}` block)
- [ ] Обработка whitespace и **indentation (off-side rule)** как в Haskell
  - Layout rule: `{` после `where`, `let`, `of`, `do`
  - Indentation определяет блоки
- [ ] Escape sequences в строках: `"\n"`, `"\t"`, `"\\"`, `"\""`
- [ ] Расширяемые числа: `440.0`, `1e-3`, `0xFF`, `0b1010`, `1_000_000` ([language-spec.md](language-spec.md) §2.4)
- [ ] Модульная система: `module Main where`, `import Lib`, `import Lib as L` ([language-spec.md](language-spec.md) §7)

**Тесты:**
- [ ] Unit tests для каждого токена
- [ ] Property-based tests: roundtrip tokenize → stringify
- [ ] Тесты на error recovery (не падать на неизвестном символе)

### 1.2. Парсер (chumsky)
**Синтаксис согласно [language-spec.md](language-spec.md)**

- [ ] Основные выражения ([language-spec.md](language-spec.md) §4):
  - Variables, literals
  - Function application: `f x y` (left-associative)
  - Infix операторы с приоритетами
- [ ] Let bindings: `let x : Type = expr in body` ([language-spec.md](language-spec.md) §4.1)
- [ ] Lambda: `\x y -> expr` (backslash, Haskell-style) ([language-spec.md](language-spec.md) §4.3)
- [ ] Pattern matching: `match expr with | Pat -> expr | ...` ([language-spec.md](language-spec.md) §6)
- [ ] If-then-else: `if cond then a else b` ([language-spec.md](language-spec.md) §4.5)
- [ ] Type annotations: `expr : Type` ([language-spec.md](language-spec.md) §3.3)
- [ ] Type definitions: `type Maybe a = Nothing | Just a` ([language-spec.md](language-spec.md) §3.4)
- [ ] Recursive functions: `let rec f n = ...` ([language-spec.md](language-spec.md) §4.3)

**FRP-конструкции ([architecture.md](architecture.md) §2.3, [language-spec.md](language-spec.md) §5):**
- [ ] Pipe operator: `|>` ([language-spec.md](language-spec.md) §4.4)
- [ ] Behavior literals и операторы
- [ ] Event literals и комбинаторы  
- [ ] Signal function syntax: `proc input -> do ...`
- [ ] Proc statements: `y <- f -< x`, `let z = ...`, `returnA -< x`

**Макросы ([architecture.md](architecture.md) §2.3, [language-spec.md](language-spec.md) §10 — reserved):**
- [ ] Macro invoke: `name!(args)` (v1.0: reserved, parse only)
- [ ] Quote: `quote! { ... }` (post-v1.0)
- [ ] Unquote: `#expr`, `#(list)` (post-v1.0)

**Тесты:**
- [ ] Парсинг всех конструкций из спецификации
- [ ] Error messages с line/column, suggestions
- [ ] Property: pretty-printed AST roundtrips

---

## Фаза 2: Type System — Система типов (5-6 недель)

**См. [architecture.md](architecture.md) §2.4 для Type формата.**

### 2.1. Основы HM-типизации
- [ ] Представление типов:
  - `Type::Var` — переменные типа `a`, `b`
  - `Type::Con` — конструкторы `Int`, `Behavior Float`
  - `Type::Arrow` — функции `a -> b`
  - `Type::Forall` — полиморфизм `forall a. a -> a`
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

### 2.4. Linear Types ([architecture.md](architecture.md) §2.4, [language-spec.md](language-spec.md) §3.8)
- [ ] `Type::Linear(Box<Type>)` — `lin τ`
- [ ] TypeFlags: `is_linear`, `is_ephemeral`, `is_send`, `is_sync`
- [ ] Linear checker: use-after-move, double-move detection
- [ ] Linear functions: `clone`, `drop` ([language-spec.md](language-spec.md) §3.8)

### 2.5. Специальные проверки
- [ ] Well-formedness типов
- [ ] Exhaustiveness pattern matching
- [ ] Execution context checking: Audio vs Control

**Тесты:**
- [ ] Positive tests: корректные программы проходят
- [ ] Negative tests: ошибки типов ловятся
- [ ] Property: inferred types are principal
- [ ] Type class resolution tests

---

## Фаза 3: Core — IR и Optimizations (3-4 недели)

### 3.1. Intermediate Representation
- [ ] ANF (A-Normal Form): все промежуточные результаты именованы
- [ ] Monomorphization: type class dictionaries → concrete functions
- [ ] Lambda lifting: free variables → parameters
- [ ] Defunctionalization для first-class functions

### 3.2. Core-to-Core оптимизации
- [ ] Inlining (heuristic-based)
- [ ] Constant folding
- [ ] Dead code elimination
- [ ] Common subexpression elimination
- [ ] Loop-invariant code motion

### 3.3. FRP-специфичные оптимизации
- [ ] Signal fusion: `map f >>> map g` → `map (f . g)`
- [ ] Event coalescing
- [ ] Static graph analysis

**Тесты:**
- [ ] Property: optimizations preserve semantics
- [ ] Benchmark suite

---

## Фаза 4: VM — Виртуальная машина (6-7 недель)

**См. [architecture.md](architecture.md) §3 для Bytecode спецификации.**

### 4.1. Bytecode Design ([architecture.md](architecture.md) §3.2)
- [ ] Variable-length instruction format
- [ ] LEB128 encoding для операндов
- [ ] Constant pool design
- [ ] Function prologue/epilogue

### 4.2. Execution Contexts ([architecture.md](architecture.md) §3.1)
- [ ] `ExecutionContext::Audio` — hard real-time
- [ ] `ExecutionContext::Control` — soft real-time
- [ ] Opcode ranges:
  - Universal: `0x00-0x9F`
  - Control only: `0xA0-0xCF`, `0xF0-0xFF`
  - Audio only: `0xD0-0xDF`
- [ ] Runtime context verification

### 4.3. VM Implementation ([architecture.md](architecture.md) §3.3)
- [ ] Stack machine core
- [ ] Call stack и frame management
- [ ] Value representation: NaN boxing
- [ ] Garbage collection (ref counting, не в audio thread!)

### 4.4. Audio Thread Architecture
- [ ] **Двухпоточная модель:**
  - Audio thread: real-time, lock-free, никаких аллокаций
  - Control thread: всё остальное (GC, JIT, I/O)
- [ ] Lock-free ring buffer для общения
- [ ] Lock-free command queue (Michael-Scott)
- [ ] Thread messages: `ParamChange`, `NoteOn`, `NoteOff`, `Meter`

### 4.5. Memory Management ([architecture.md](architecture.md) §3.3)
- [ ] Region-based allocation для audio frames
- [ ] Object pools для часто используемых структур
- [ ] Linear types runtime checks (debug builds)
- [ ] Double/triple buffering для аудио

### 4.6. Error Handling ([architecture.md](architecture.md) §3.6)
- [ ] Audio thread: soft failure (Clip, Silence, Continue)
- [ ] Control thread: exceptions (TRY, CATCH, THROW)

**Тесты:**
- [ ] VM unit tests (each opcode)
- [ ] Stress tests (long-running)
- [ ] Memory leak tests (valgrind/miri)
- [ ] Real-time safety tests (no syscalls in audio thread)

---

## Фаза 5: Audio Backend — Низкоуровневое аудио (4-5 недель)

### 5.1. Audio Abstraction Layer
- [ ] Trait `AudioBackend` с методами `start`, `stop`, `sample_rate`

### 5.2. Backend Implementations
- [ ] **CPAL** (cross-platform fallback)
  - Linux: ALSA через CPAL
  - macOS: CoreAudio через CPAL
- [ ] **JACK** (Linux, опционально для v1.0)
  - Низкая latency
  - Межприложенная маршрутизация

### 5.3. Real-time Guarantees ([architecture.md](architecture.md) Appendix C)
- [ ] `SCHED_FIFO` priority для audio thread
- [ ] `mlockall()` для предотвращения swapping
- [ ] CPU affinity (dedicated cores)
- [ ] No allocation in audio callback
- [ ] Lock-free всё

### 5.4. Audio Primitives ([architecture.md](architecture.md) §3.5)
- [ ] Phasor (фазовый аккумулятор)
- [ ] Sine oscillator (table-based): `OSC_SIN`
- [ ] Audio I/O: `audioIn` / `audioOut`
- [ ] Gain / pan

**Тесты:**
- [ ] Тест tone generation (sin 440Hz)
- [ ] Latency measurement
- [ ] Glitch detection (zero crossings, DC offset)

---

## Фаза 6: IFRP — Инкрементальное FRP (8-10 недель)

**См. [architecture.md](architecture.md) §4.3 для StatefulNodes.**

### 6.1. Term Labeling
- [ ] Присвоение уникальных ID каждому stateful узлу
- [ ] AST annotation pass с `node_id`
- [ ] Source location → label mapping
- [ ] Explicit labels: `#[stable("lfo")]`

### 6.2. State Representation ([architecture.md](architecture.md) §4.3)
- [ ] `StateMap: node_id -> Box<dyn AnyState>`
- [ ] Stateful node descriptors: `StatefulNode`
- [ ] `StatefulType`: Oscillator, ADSR, DelayLine, Biquad, Ref
- [ ] Type-safe state serialization
- [ ] State versioning для миграции

### 6.3. Graph Diff Algorithm
- [ ] AST comparison: structural matching
- [ ] Label mapping: old -> new
- [ ] Detection: preserved / modified / added / removed
- [ ] Topological order preservation

### 6.4. State Migration
- [ ] Copy state для preserved nodes
- [ ] Initialize new nodes с defaults
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
- [ ] Cranelift integration
- [ ] Tiered compilation:
  - Tier 0: Interpreter + profiling
  - Tier 1: Cranelift quick compile
  - Tier 2: Cranelift optimize

### 7.2. Profiling
- [ ] Basic block counters
- [ ] Execution time tracking
- [ ] Hotspot detection

### 7.3. Compilation Triggers
- [ ] Call count > threshold
- [ ] Execution time > 10% frame budget
- [ ] Explicit annotation: `#[jit]`
- [ ] Stability: code unchanged > 5 seconds

### 7.4. Optimizations ([architecture.md](architecture.md) §3.5 SIMD)
- [ ] Loop unrolling для аудио блоков
- [ ] SIMD vectorization: `VEC_F32x4_ADD`, `VEC_F32x8_ADD`
- [ ] Function inlining
- [ ] Constant propagation

### 7.5. Code Replacement
- [ ] Atomic pointer swap
- [ ] Version counters
- [ ] Safe deallocation старого кода

**Тесты:**
- [ ] Correctness: JIT == interpreter
- [ ] Performance benchmarks
- [ ] Compilation time limits

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
```
- [ ] Direct Form II
- [ ] State preservation
- [ ] Frequency response accuracy

### 8.4. FRP Combinators
```
-- Behaviors
pure    :: a -> Behavior a
liftA2  :: (a -> b -> c) -> Behavior a -> Behavior b -> Behavior c
integral:: Behavior Float -> Behavior Float

-- Events
never   :: Event a
merge   :: Event a -> Event a -> Event a
hold    :: a -> Event a -> Behavior a
accum   :: a -> Event (a -> a) -> Behavior a

-- Signal Functions
arr     :: (a -> b) -> SF a b
(>>>)   :: SF a b -> SF b c -> SF a c
```

### 8.5. I/O
```
readWav  :: FilePath -> Audio (Behavior [Float])
writeWav :: FilePath -> Behavior [Float] -> Audio ()
midiIn   :: Audio (Event MidiMessage)
audioOut :: Behavior [Float] -> Audio ()
```

**Тесты:**
- [ ] Unit tests для каждого примитива
- [ ] Property-based: filter frequency response
- [ ] Golden tests: compare с reference

---

## Фаза 9: Effects — Эффекты обработки (5-6 недель)

### 9.1. Delay Effects
- [ ] `delay :: Time -> Feedback -> Mix -> Behavior Float -> Behavior Float`
- [ ] Multi-tap delay
- [ ] Ping-pong delay

### 9.2. Algorithmic Reverb
- [ ] `freeVerb :: RoomSize -> Damping -> Width -> DryWet -> ...`
- [ ] All-pass и comb фильтры
- [ ] Schroeder/Moorer алгоритмы

### 9.3. Dynamics
- [ ] `compressor :: Threshold -> Ratio -> Attack -> Release -> ...`
- [ ] Limiter
- [ ] Gate
- [ ] Sidechain input

### 9.4. Modulation Effects
- [ ] Chorus
- [ ] Flanger
- [ ] Phaser

### 9.5. Distortion
- [ ] Soft clip (tanh)
- [ ] Hard clip
- [ ] Bit crusher

---

## Фаза 10: CLI и Developer Tools (3-4 недели)

### 10.1. CLI Interface
```bash
linen run file.ln [--jit]
linen compile file.ln -o out.lnc
linen repl
linen check file.ln
```

### 10.2. REPL
- [ ] Expression evaluation
- [ ] :type команда
- [ ] :reload для hot reload
- [ ] История, tab completion

### 10.3. Package Manager (базовый)
- [ ] `linen.toml` формат
- [ ] Dependencies
- [ ] `linen install`

### 10.4. Debugger / Profiler
- [ ] Signal graph visualization
- [ ] Real-time scope
- [ ] CPU usage per node

---

## Фаза 11: macOS Support (3-4 недели)

- [ ] CoreAudio backend через CPAL
- [ ] Universal binary (Intel + Apple Silicon)
- [ ] CI/CD для macOS

---

## Фаза 12: Documentation и Release (3-4 недели)

- [ ] Language tutorial
- [ ] Standard library reference
- [ ] Cookbook с примерами
- [ ] Video tutorials (optional)
- [ ] Website и playground

---

## Итоговая воронка задач

| Фаза | Длительность | Основные результаты |
|------|--------------|---------------------|
| 0 | 2-3 недели ✅ | Инфраструктура, CI, архитектура |
| 1 | 3-4 недели | Лексер, парсер, CST → AST |
| 2 | 5-6 недель | Type checker, Linear types |
| 3 | 3-4 недели | IR, оптимизации |
| 4 | 6-7 недель | VM с Audio/Control split |
| 5 | 4-5 недель | Audio backend |
| 6 | 8-10 недель | IFRP, Hot reload |
| 7 | 6-8 недель | JIT (Cranelift) |
| 8 | 6-7 недель | Core stdlib |
| 9 | 5-6 недель | Effects |
| 10 | 3-4 недели | CLI, REPL |
| 11 | 3-4 недели | macOS |
| 12 | 3-4 недели | Документация |

**Общая длительность:** ~12-18 месяцев

---

## MVP Definition (v0.1 — 3 месяца)

Минимальный viable product:

- Парсер + Type checker (без Linear types)
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
| Real-time гарантии на macOS | Средняя | Среднее | Использовать CPAL, fallback к buffering |
| Cranelift интеграция | Средняя | Среднее | Сначала interpreter-only |

---

## Метрики успеха v1.0

- [ ] Latency < 10ms на Linux с JACK
- [ ] CPU usage < 10% для 10 голосов полифонии
- [ ] Hot reload < 100ms
- [ ] Стабильность: 24 часа работы без сбоев
- [ ] Тестовое покрытие > 80%

