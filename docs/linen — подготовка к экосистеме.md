# linen — подготовка к модульной и пакетной экосистеме

> Дополнение к `SPEC.md`. Документ описывает, какие свойства нужно сохранить на ранних этапах, чтобы позднее добавить модули, библиотеки, инструменты, пакетный менеджер, desktop-плагины и embedded-устройства.
> Это **не задание реализовывать модульную систему или пакетный менеджер сейчас**.
> Любое изменение текущих контрактов сначала согласовывается с основной спекой.

- Версия: 0.2
- Дата: 2026-11-15
- Статус: архитектурные ограничения и будущая дорожная карта; ближайшие решения интегрированы в `SPEC.md` 0.2.1
- Основной документ: `SPEC.md`
- Текущая реализация: этапы 0, 1 и корректирующий этап 1A завершены; фактическая приёмка находится в `docs/development/stages/stage-01a-ecosystem-foundation.md`
- Основной принцип разработки: человек пишет ядро; агенты — тесты, fixtures, генераторы и явно разрешённый бойлерплейт

## 0. Главный вывод

Для будущей экосистемы сейчас нужно заложить не registry, CLI пакетного менеджера или сложные абстракции, а несколько дешёвых на раннем этапе свойств:

1. Компилятор умеет различать источники через `SourceId`, даже если пока источник всегда один.
2. Parser, resolver, type checker, lowering и VM не обращаются к файловой системе самостоятельно.
3. Имена и символы не сводятся к строкам и глобальному namespace.
4. Узлы IR сохраняют происхождение: исходный файл, функцию и source span.
5. Builtins имеют структурированные метаданные, а не разбросаны по unrelated `match`.
6. Компиляция детерминирована и не зависит от сети, системного времени, случайного состояния и абсолютных путей.
7. Возможности программы и её ресурсные требования можно будет определить статически.
8. Исходные библиотеки, законченные инструменты и device bytecode являются разными артефактами.
9. Байткод не считается стабильным ABI раньше времени, но любой сохраняемый формат имеет явную версию.
10. Audio runtime остаётся отделённым от compiler frontend и host I/O.

Если эти свойства сохранятся, экосистему можно будет добавить постепенно без полного переписывания языка.

## 1. Видение экосистемы

Вокруг linen могут появиться разные виды переиспользуемых результатов:

```text
linen language + compiler + VM
            │
            ├── Source libraries
            │     envelopes, filters, oscillators, sequencing helpers,
            │     musical patterns, mixing utilities
            │
            ├── Instruments
            │     synths, drum machines, effects, generative machines
            │
            ├── Presets
            │     parameter values without new DSP code
            │
            ├── Desktop products
            │     standalone instruments, VST3/CLAP plugins, workbench
            │
            └── Embedded products
                  verified device packages executed by a small VM
```

Экосистема должна позволять:

- публиковать функцию или набор функций как библиотеку;
- использовать библиотеку без копирования исходника;
- собрать законченный инструмент с параметрами и пресетами;
- воспроизвести старую сборку через lockfile;
- проверить, какие возможности языка нужны пакету;
- отклонить пакет, не поддерживаемый конкретным устройством;
- исполнять один и тот же инструмент на desktop и embedded target с эквивалентной семантикой;
- диагностировать ошибку внутри зависимости с нормальным исходным span;
- обновлять компилятор без незаметного изменения старых патчей.

### 1.1. Что является ценностью экосистемы

Не количество пакетов само по себе, а:

- качественные небольшие DSP-компоненты;
- понятные музыкальные абстракции;
- работающие инструменты;
- переносимость;
- воспроизводимость;
- доверие к пакетам;
- отсутствие скрытых зависимостей от конкретной ОС.

### 1.2. Главное ограничение

**Пакетный менеджер не должен становиться вторым языком или второй операционной системой.**

Не нужны:

- install scripts;
- arbitrary shell commands;
- native dynamic libraries в обычных пакетах;
- сетевые запросы во время DSP graph construction;
- скрытая кодогенерация со side effects;
- автоматический запуск кода при установке;
- неограниченный доступ библиотеки к host API.

Основной формат экосистемы — **декларативные данные и исходный код linen**.

## 2. Разделение артефактов

Следующие понятия нельзя смешивать.

| Артефакт | Содержимое | Где исполняется/используется |
|---|---|---|
| Source library | Функции, типы, константы, документация | Компилируется вместе с программой |
| Project | Локальные модули, зависимости, main/instrument | Desktop compiler |
| Instrument | Entry point, параметры, presets, resource/capability metadata | Desktop instrument или plugin |
| Preset | Значения параметров и ссылка на совместимый instrument | Без изменения графа |
| Device package | Проверенный bytecode, VM version, capabilities, resource usage | Embedded VM |
| Native host extension | OS-specific машинный код | Только отдельный privileged формат, не обычный пакет |

### 2.1. Source library

Основной способ повторного использования кода. Библиотека не поставляет отдельный `.so`/`.dll`, а компилируется в граф текущей программы.

Последствия:

- нет стабильного binary ABI для каждой библиотеки;
- функции могут inline’иться и специализироваться;
- stateful-узлы создаются по обычным правилам вызова;
- публичная совместимость определяется исходным API;
- lockfile должен фиксировать точное содержимое библиотеки.

### 2.2. Instrument

Законченный инструмент — не просто библиотека функций. У него есть:

- один явный entry point;
- входы/выходы;
- список внешних параметров;
- presets;
- требуемые capabilities;
- resource limits;
- версия языка;
- тестовый рендер;
- документация по управлению.

### 2.3. Device package

Устройству не нужно получать весь исходный проект и компилятор. Предпочтительный путь:

```text
source project + lockfile
          ↓ desktop compiler
verified instrument artifact
          ↓ target-specific codegen/packaging
device package
          ↓ verifier on device
linen embedded VM
```

Device package отдельно версионируется и проверяется. Исходная библиотека не должна автоматически считаться пригодной для устройства.

## 3. Целевая модульная система

Точный синтаксис `use`, `export` и module paths будет зафиксирован отдельным ADR перед реализацией. Сейчас не нужно добавлять его в parser.

### 3.1. Семантические требования

- Модуль имеет стабильный логический идентификатор, не зависящий от абсолютного пути на машине.
- Каждый модуль имеет собственный namespace.
- Символы по умолчанию приватны; публичность явная.
- Локальные имена не попадают автоматически в importer.
- Импорт создаёт однозначную ссылку на модуль/символ, а не текстовое включение файла.
- Циклы модулей запрещены до отдельного решения; отсутствие циклов упрощает resolution и graph construction.
- Порядок файлов на файловой системе не влияет на результат.
- Один модуль не должен компилироваться дважды из-за двух импортов.
- Diamond dependency не создаёт неявное дублирование глобального состояния.
- Состояние возникает только при конкретном вызове функции/создании узла, а не от самого факта импорта.
- Модули не имеют скрытых top-level side effects.
- Публичные сигнатуры функций остаются явными.
- Diagnostics показывают логическое имя модуля и исходный span внутри зависимости.

### 3.2. Локальные модули до пакетов

Правильная последовательность:

```text
single file
   ↓
несколько файлов одного project
   ↓
локальные path packages
   ↓
lockfile и content hashes
   ↓
remote registry
```

Нельзя начинать с registry и затем пытаться выяснить, что означает имя модуля.

### 3.3. Зарезервированный синтаксический простор

В `SPEC.md` 0.2.0 наряду с `mod` и `use` дополнительно зарезервированы:

```text
export
package
from
as
```

Решение принято 2026-11-15 и реализовано на этапе 1A. Это только лексическое резервирование, а не реализация синтаксиса модулей или пакетов. Tokens, lexer tests и compatibility fixtures обновлены; grammar declarations/imports не добавлялись. Нельзя резервировать новые слова в коде без отражения в спеке.

## 4. Что заложить в архитектуру сейчас

### 4.1. `SourceId`, `SourceMap` и многофайловые spans

Наиболее дешёвое и полезное раннее решение.

Концептуальная модель:

```text
SourceId
  компактный ID, уникальный внутри CompileSession

SourceFile
  id: SourceId
  logical_name: String        # например "main.ln" или "library/osc.ln"
  text: String
  line_index: LineIndex

SourceSpan
  source: SourceId
  start: usize                # byte offset
  end: usize                  # exclusive

SourceMap
  sources: Vec<SourceFile>
```

Требования:

- На этапе 1A простой `check` создаёт один `SourceFile`; synthetic tests используют несколько, а Beep сохраняет ту же модель. Никакой видимой модульной функциональности из этого не следует.
- На 1A offsets остаются `usize`, чтобы не менять существующий source-size contract. Для будущих serialized spans ширина и overflow rules фиксируются отдельным форматом/ADR.
- `SourceId` стабилен только внутри создавшей его `SourceMap` и не является публичным глобальным идентификатором пакета. Независимые карты могут выдавать равные индексы; ID используется только с исходной картой или её идентичной копией. Unknown source означает отсутствующий в карте индекс, а не обнаруженный foreign ID с совпадающим индексом.
- Diagnostics принимает `SourceSpan`, а не просто offsets, которые потом невозможно связать с файлом.
- Все этапы AST → Typed AST → IR сохраняют `SourceSpan`.
- `logical_name` используется в diagnostics; абсолютный путь не зашивается в результаты компиляции.
- `line_index` корректно обрабатывает LF и CRLF.
- Parser получает уже загруженный текст и `SourceId`; он не открывает файлы.
- Не нужно заранее создавать network loader, virtual filesystem или plugin-based source providers.

**Почему сейчас:** менять одну файловую модель на многофайловую после появления AST, Typed AST, IR, bytecode debug info и сотен snapshots будет значительно дороже.

### 4.2. Явный `CompileSession`

Компиляция должна иметь владельца контекста:

```text
CompileSession
  sources: SourceMap
  config: CompileConfig
  diagnostics: DiagnosticSink
  symbols: SymbolStorage
```

Допустимы отдельные структуры вместо одной большой, если владение явное.

Запрещено:

- глобальные mutable interner;
- глобальный текущий файл;
- глобальный random;
- чтение env в середине type checking;
- обращение к current working directory из parser/lowering;
- скрытое кэширование скомпилированных модулей между sessions без явного ключа.

`compile_source(...)` остаётся простым facade: внутри создаётся session с одним источником. Его outcome должен сохранять `SourceMap` вместе с AST/diagnostics, чтобы returned `SourceId` всегда можно было разрешить; при session-based API caller удерживает сам session. В будущем `compile_project(...)` использует ту же модель с несколькими источниками.

На этапе 1A fail-fast diagnostics возвращаются через `Result`, поэтому пустой `DiagnosticSink` и `SymbolStorage` заранее не создаются. Они появляются вместе с реальными recovery и resolver. Подробный контракт находится в ADR 0012 и stage 1A.

### 4.3. Resolver отделён от загрузки

Name resolution отвечает на вопрос «на какое определение ссылается это имя?». Он не отвечает на вопрос «откуда скачать файл?».

Границы:

```text
Source loading / future module loading
              ↓ готовые SourceFile + logical module IDs
        Name resolver
              ↓ resolved definitions
         Type checker
```

До модульной системы не нужен общий `ModuleLoader` trait с десятью реализациями. Достаточно не делать файловый I/O в resolver и не использовать path как ключевая семантика имени.

### 4.4. Символы и идентичность определений

Различать:

- `BindingId` — конкретная локальная/глобальная привязка;
- `FunctionId` — определение пользовательской функции;
- `BuiltinId` — встроенный узел;
- будущий `ModuleId` — модуль;
- будущий `PackageId` — пакет;
- строковое имя — только часть diagnostics и lookup.

Требования:

- Resolved AST ссылается на ID, а не ищет имя повторно по строке.
- ID уникален в рамках явно определённой области.
- Нельзя использовать адрес Rust-структуры как стабильную идентичность.
- Нельзя использовать случайный UUID, влияющий на выход компиляции.
- Будущий fully-qualified symbol строится из логического module/package ID и локального имени, а не из абсолютного пути.
- Два одинаковых имени в разных модулях не должны конфликтовать сами по себе.

### 4.5. Происхождение узлов в IR и bytecode debug info

Узел IR должен сохранять не только span, но и контекст происхождения:

```text
NodeOrigin
  primary: SourceSpan
  inlined_from: optional chain of call spans   # когда появятся функции/inlining
```

После появления пакетов через `SourceId` можно будет показать:

```text
error in generated low-frequency oscillator
  → node originated in package curves, module envelopes
  → inlined from instrument main.ln
```

Требования:

- origin не влияет на звук;
- origin можно опускать в compact production bytecode;
- debug artifact должен позволять связать runtime error/probe с исходником;
- после optimization нельзя оставлять заведомо ложный span;
- при объединении узлов сохраняется либо точный origin, либо явный merged/unknown origin по правилам ADR.

### 4.6. Builtins как метаданные, а не разрозненная магия

Для каждого builtin постепенно должны быть доступны:

```text
BuiltinDescriptor
  id
  name
  signature
  purity/determinism class
  statefulness
  required capabilities
  resource model
  documentation name
```

На раннем этапе это может быть обычная статическая таблица и связанные проверки, а не универсальная plugin system.

Зачем:

- одинаковая сигнатура используется type checker и документацией;
- capability analysis не требует повторного ручного перечисления всех узлов в пяти местах;
- device compiler может отклонить неподдерживаемый узел;
- будущий workbench может показать параметры builtin;
- тесты могут проверить полноту descriptor для каждого IR op.

Не нужно делать builtins динамически загружаемыми Rust-библиотеками.

### 4.7. Детерминированная компиляция

Одинаковые логические входы должны давать одинаковый компиляторный результат:

```text
source files
+ dependency contents
+ language/config version
+ explicit render seed
= deterministic result
```

Не должны влиять:

- абсолютный путь проекта;
- порядок directory listing;
- hash randomization;
- системное время;
- hostname;
- network availability;
- locale;
- текущий пользователь;
- недетерминированный порядок обхода `HashMap`.

Правила:

- перед сериализацией или diagnostics сортировать по явному стабильному ключу;
- seed передавать через config;
- не включать timestamp в patch artifact по умолчанию;
- error ordering должен быть детерминирован;
- tests сравнивают повторные компиляции в одном process и в разных processes.

### 4.8. Случайность и стабильная идентичность

Random/noise не должен зависеть от случайного `NodeId`, который изменился после inlining или optimization.

Возможные будущие решения:

- явный seed у каждого noise-узла;
- стабильный `RandomStreamId`, выведенный из логического определения и экземпляра вызова;
- явный seed в `RenderConfig` плюс стабильное stream identity.

Выбор фиксируется ADR до реализации noise. Запрещено делать звук зависимым от порядка обхода графа, memory address или номера инструкции.

### 4.9. Параметры как отдельная модель

Для будущих UI, плагинов и hardware-ручек внешний параметр должен иметь метаданные:

```text
ParameterDescriptor
  stable parameter ID
  display name
  type
  default
  min/max или domain
  unit
  smoothing policy
  group
  documentation
```

Сейчас не добавлять annotation syntax или UI. При проектировании будущих `control`, instrument entry point и IR следует оставить место для такого descriptor.

Требования будущей модели:

- parameter ID стабилен при переименовании display name;
- preset ссылается на parameter ID, а не на позицию ручки;
- изменение smoothing не перестраивает весь граф;
- параметры имеют явную частоту обновления и правила квантования;
- host automation и hardware controls используют одну модель;
- параметры не вызывают allocations в audio steady-state.

### 4.10. Compiler, instrument и runtime остаются отдельными

```text
Frontend/compiler         может быть большим, использует filesystem на host boundary
        ↓
Verified program          не зависит от CLI
        ↓
VM core                   не знает о WAV, VST, display, GPIO
        ↓
Backend                   WAV / desktop audio / plugin / embedded
```

Запрещено:

- импортировать CLI в VM;
- передавать `stdout` в DSP;
- вызывать host plugin API из языкового выражения;
- читать WAV или MIDI файлы из фильтра;
- связывать `RenderConfig` с конкретным windowing toolkit;
- требовать ОС для базовой верификации bytecode.

Это уже соответствует основной спеке и должно сохраняться при добавлении каждого backend.

## 5. Capabilities и ресурсные требования

Для desktop plugin, offline renderer и маленького устройства одна и та же программа может иметь разную допустимость. Это должно определяться явно, а не обнаруживаться через исчерпание памяти или щелчки.

### 5.1. Не смешивать три вида версий

1. **Language version** — синтаксис и семантика исходника.
2. **Bytecode/VM version** — набор инструкций и правил исполнения.
3. **Package version** — версия конкретной библиотеки или инструмента.

Дополнительно существует **compiler version**, которая сама по себе не должна входить в языковую семантику.

Нельзя считать, например, `package 2.0` автоматически совместимым с `VM 2.0` только из-за совпадения чисел.

### 5.2. Language features

Описывают необходимые возможности frontend:

```text
functions
records
variants
events
feedback
...
```

Они нужны для проверки исходной совместимости и сообщений вида «этот пакет требует variants, которые текущий compiler не поддерживает».

### 5.3. Runtime capabilities

Описывают то, что требуется исполнителю:

```text
core-audio
control-input
event-input
seeded-random
delay
sample-playback
fft
multichannel
...
```

Точный список появляется вместе с узлами. Не создавать сейчас пустой универсальный enum со всеми будущими возможностями.

Каждый новый IR op обязан получить capability classification до добавления device/limited targets.

### 5.4. Target descriptor

Будущая концептуальная модель:

```text
TargetDescriptor
  target name
  supported VM/bytecode versions
  supported runtime capabilities
  allowed sample rates
  maximum nodes
  maximum state bytes
  maximum voices
  maximum delay memory
  maximum input/output channels
  realtime constraints
```

До появления второго target не нужен сложный target plugin API. Но ограничения не следует зашивать в отдельные условия `if is_embedded` по всему компилятору.

### 5.5. Resource analysis

Для device package желательно статически вычислять или ограничивать:

- число узлов;
- число state slots;
- объём state;
- audio register buffers;
- delay memory;
- максимальное число voices;
- размер event queues;
- число входов/выходов;
- необходимость dynamic graph changes;
- наличие недетерминированных или неподдерживаемых операций.

Если точное значение неизвестно, должен быть доказуемый верхний предел. Для restricted target программа без доказуемого лимита отклоняется с diagnostic, а не исполняется до OOM.

Resource analysis не обязан появиться в Beep. Сейчас нужно только не делать ресурсы скрытыми в глобальных runtime-сервисах.

## 6. Будущая пакетная система

### 6.1. Последовательность появления

1. Многофайловый project.
2. Модули и явные exports.
3. Локальные path packages.
4. Manifest и API visibility.
5. Lockfile с точными versions/content hashes.
6. Воспроизводимые offline builds.
7. Локальный package cache.
8. Read-only remote registry/install.
9. Publishing, namespaces и ownership.
10. Device artifacts и target-specific distribution.

Publishing без lockfile и воспроизводимых builds создаст экосистему, в которой старые инструменты невозможно надёжно восстановить.

### 6.2. Manifest — будущий контракт

Иллюстративный, **не разрешённый к текущей реализации** пример:

```toml
format = 1
name = "examples/curves"
version = "0.1.0"
kind = "library"
entry = "src/lib.ln"
language = ">=0.4, <0.5"
capabilities = ["core-audio"]

[dependencies]
# Точный dependency syntax фиксируется отдельным ADR.
```

В окончательном manifest должны быть однозначно определены:

- package kind;
- логическое имя;
- версия;
- entry point;
- language compatibility;
- dependencies;
- capabilities;
- license;
- exported API;
- assets и их пределы, если assets поддерживаются;
- инструментальные параметры для instrument package;
- digest/format rules для распространения.

Не нужно выбирать TOML и реализовывать parser сейчас. Сейчас важно, чтобы эти данные не становились неявными соглашениями в исходниках.

### 6.3. Lockfile

Lockfile должен фиксировать:

- точные resolved package versions;
- content hash каждого пакета;
- транзитивные зависимости;
- resolution order или достаточные данные для её воспроизведения;
- language/toolchain compatibility constraints;
- asset hashes;
- device artifact inputs, если собирается device package.

Правила:

- lockfile не редактируется вручную как обычный config;
- повторный install с тем же lockfile не обращается к mutable «latest»;
- offline build возможен при заполненном cache;
- отсутствие содержимого с нужным hash — явная ошибка;
- registry metadata не изменяет уже зафиксированный resolution;
- два проекта с одинаковым lockfile и source inputs получают одинаковый dependency graph.

### 6.4. Content addressing

Версия вида `1.2.0` удобна людям, но не доказывает содержимое. Для хранения и проверки нужен cryptographic content hash.

Разделяются:

```text
package identity: namespace/name
release identity: namespace/name@version
artifact identity: content hash
```

Registry не должен молча заменять содержимое уже выпущенной release identity. Yanking может запрещать новое разрешение, но не должен незаметно менять старый lockfile build.

### 6.5. Безопасность пакетов

Обычный source package:

- не имеет install script;
- не исполняет shell;
- не получает сетевой доступ;
- не читает произвольные файлы;
- не загружает native library;
- не меняет compiler options проекта;
- не может отключить verifier или resource limits;
- не выполняет top-level host side effects;
- компилируется с теми же limits, что и локальный код.

При распаковке будущего package archive проверяются:

- отсутствие absolute paths и `..`;
- отсутствие symlink/hardlink escapes;
- лимит compressed/uncompressed size;
- лимит числа файлов;
- duplicate names и case collisions;
- допустимость UTF-8/имён;
- digest содержимого;
- отсутствие неожиданных executable/native payloads.

### 6.6. Подписи и владение именами

Это registry-этап, не ранняя архитектура языка.

Понадобятся:

- namespace ownership;
- механизм подписи или подтверждения publisher;
- audit metadata;
- политика yank;
- защита от dependency confusion;
- понятная обработка удалённых/скомпрометированных пакетов.

До registry не нужно реализовывать собственную криптографию. Для локальных пакетов достаточно path identity и позднее content hashes.

## 7. Стандартная библиотека и dogfooding

Не весь DSP должен быть builtin.

### Builtin

Подходит для:

- базовых операций VM;
- примитивов с собственным состоянием;
- операций, требующих специального codegen;
- узлов, имеющих чёткие capability/resource contracts.

### Source library

Подходит для:

- envelope helpers;
- композиций фильтров;
- supersaw и подобных осцилляторов;
- mixing utilities;
- музыкальных паттернов;
- эквалайзеров, собранных из базовых фильтров;
- стандартных voice/instrument abstractions.

Когда модульная система появится, часть стандартной библиотеки следует написать на самой linen. Это проверит:

- exports;
- документацию;
- функциональную композицию;
- diagnostics внутри зависимости;
- package tests;
- производительность после inlining;
- независимость состояния вызовов.

Не следует на старте разделять компилятор и stdlib на множество репозиториев. Сначала stdlib может жить в том же репозитории и использовать тот же модульный механизм, что и внешние библиотеки.

## 8. Совместимость API и совместимость звука

Это разные свойства.

### API compatibility

Означает, что пакет можно скомпилировать без изменения вызовов:

- имена;
- типы;
- arity;
- exports;
- semantics аргументов.

### Render compatibility

Означает, что патч сохраняет ожидаемый звук:

- начальное состояние;
- точность;
- random streams;
- DSP equations;
- event ordering;
- parameter smoothing;
- optimizer behavior.

Изменение библиотеки с `0.1` до `0.2` может сохранить API, но заметно изменить инструмент. Поэтому:

- инструмент фиксирует dependencies через lockfile;
- presets дополнительно сохраняют ожидаемый engine/package compatibility;
- тестовый reference render входит в пакет;
- изменение DSP algorithm не маскируется под «internal refactor»;
- migration presets описывается отдельно;
- побайтовая идентичность на всех платформах не обещается без специального deterministic profile;
- числовые tolerances и критерии музыкальной эквивалентности задаются явно.

## 9. Дорожная карта экосистемы

### E0. Текущая подготовка — во время ранних этапов SPEC.md

**Цель:** не создать препятствия для модулей и пакетов.

**Человек в production-коде:**

- вводит `SourceId`, `SourceFile`, `SourceMap`, `SourceSpan`;
- обеспечивает явное владение sources/config/diagnostics;
- убирает файловый I/O из внутренних compiler stages;
- сохраняет source origin в Typed AST и IR;
- использует resolved IDs вместо повторного строкового resolution;
- поддерживает детерминированный порядок diagnostics;
- по мере появления builtins добавляет descriptor metadata;
- не стабилизирует bytecode преждевременно.

**Агенту можно:**

- написать tests для `SourceMap`, line index и multi-source diagnostics;
- проверить отсутствие absolute paths в diagnostic artifacts;
- добавить deterministic compilation tests;
- подготовить tests на порядок diagnostics;
- проверить сохранение spans через compiler stages;
- сделать read-only review зависимостей модулей.

**DoD:**

- один файл работает через ту же SourceMap-модель, которая позднее поддержит несколько;
- test can создать два независимых SourceFile и однозначно отличить их diagnostics;
- compiler stages не открывают файлы;
- повторная компиляция не зависит от hash randomization;
- нет module/package CLI и пустых abstraction layers;
- Beep остаётся настолько же простым с точки зрения пользователя.

### E1. Многофайловый project

Появляется только после стабильных пользовательских функций и достаточной зрелости diagnostics.

**Возможности:**

- несколько SourceFile;
- логические module IDs;
- явные imports/exports по утверждённому синтаксису;
- cycle detection;
- module visibility;
- единая SourceMap;
- diagnostics через границы файлов.

**DoD:**

- diamond imports не дублируют неявное состояние;
- cycle даёт diagnostic с полной цепочкой;
- одинаковые имена в разных модулях не конфликтуют;
- private symbol недоступен importer;
- ошибка в imported file показывает правильный source;
- результат не зависит от directory listing;
- single-file programs прошлых этапов не меняются.

### E2. Локальные пакеты

**Возможности:**

- package manifest по ADR;
- library/instrument package kinds;
- path dependencies;
- публичный API;
- dependency graph;
- локальный test runner пакета;
- package reference renders.

**DoD:**

- один package используется двумя проектами без copy-paste;
- package tests запускаются изолированно;
- package не выполняет install/host side effects;
- duplicate dependency versions обрабатываются по явному правилу;
- dependency errors содержат package/module spans;
- отсутствует network requirement.

### E3. Lockfile и воспроизводимые сборки

**Возможности:**

- deterministic resolution;
- exact versions и content hashes;
- offline cache;
- immutable resolution;
- reproducible instrument artifact inputs;
- diff dependency graph.

**DoD:**

- одинаковый lockfile даёт одинаковый graph на повторной сборке;
- очистка registry не меняет уже разрешённый build при наличии cache;
- изменение содержимого без изменения version обнаруживается hash mismatch;
- offline success/failure явно диагностируется;
- dependency update является отдельной операцией, а не скрытым следствием build.

### E4. Инструменты и пресеты как пакеты

**Возможности:**

- instrument entry point;
- parameter descriptors;
- presets по stable parameter IDs;
- capability declaration/check;
- resource summary;
- reference audio tests;
- companion desktop editor;
- экспорт device candidate artifact.

**DoD:**

- пресет не зависит от порядка параметров;
- неподдерживаемая capability обнаруживается до audio execution;
- resource limit нарушение даёт понятный diagnostic;
- instrument можно восстановить из lockfile;
- reference render выявляет изменение DSP;
- один instrument работает в headless test и desktop backend.

### E5. Registry и публикация

**Возможности:**

- read-only install из registry;
- namespaces;
- publishing;
- ownership;
- immutable releases;
- yank policy;
- package archive security;
- подпись/audit metadata.

**DoD:**

- dependency confusion сценарии покрыты тестами;
- malicious archives безопасно отклоняются;
- registry не исполняет package code;
- release content нельзя незаметно заменить;
- cached offline install воспроизводим;
- публикация не требуется для локальной разработки.

### E6. Embedded distribution

**Возможности:**

- target descriptors;
- versioned device packages;
- target bytecode verification;
- static resource limits;
- capability checks;
- compact debug info;
- host-to-device transfer;
- device compatibility tests.

**DoD:**

- устройство не требует compiler frontend;
- повреждённый/неподдерживаемый package отклоняется;
- превышение памяти обнаруживается до загрузки;
- audio state остаётся в заданных пределах;
- host/device differential suite проходит;
- обновление VM имеет явную migration/compatibility policy.

## 10. Тестовая стратегия экосистемы

### 10.1. Что тестировать уже на E0

- `SourceMap` с одним и несколькими synthetic sources;
- byte spans, line/column, LF и CRLF;
- diagnostics всегда указывают на правильный `SourceId`;
- одинаковый `logical_name` в разных synthetic sources не смешивает spans;
- parser/type checker работают с in-memory source без filesystem;
- повторная компиляция даёт одинаковый порядок diagnostics;
- абсолютный путь не попадает в semantic artifact;
- `SourceId` не влияет на звук;
- origin каждого IR node ведёт к валидному span;
- `HashMap` iteration не меняет codegen/output ordering.

### 10.2. Модульные тесты на E1

- import public symbol;
- reject private symbol;
- namespace collision;
- explicit alias;
- unused/duplicate imports по контракту;
- cycle of length 1, 2 и больше;
- diamond dependencies;
- repeated import;
- same filename in different directories;
- ошибка внутри зависимости;
- вызов stateful function из двух модулей;
- отсутствие side effects от неиспользуемого import;
- deterministic resolution при разном порядке перечисления файлов.

### 10.3. Пакетные тесты

- minimal valid package;
- missing/invalid manifest fields;
- unsupported language version;
- dependency cycle между packages;
- diamond version resolution;
- two versions одной библиотеки;
- lockfile immutability;
- content hash mismatch;
- corrupted cache;
- offline install;
- package with unexpected native executable;
- archive traversal, symlink, duplicate/case-colliding paths;
- decompression/resource bomb limits;
- reproducible reference render;
- diagnostics внутри transitively imported package.

### 10.4. Capability и device tests

- каждый IR op имеет capability classification;
- каждый supported target имеет явный feature matrix;
- unsupported feature отклоняется до codegen/loading;
- resource estimate не ниже фактически выделенного state;
- delay memory учитывает sample rate;
- voice limit не может быть превышен;
- malformed device package не вызывает panic;
- VM version mismatch даёт отдельный diagnostic;
- debug info stripping не меняет звук;
- host/device block partitions сохраняют семантику;
- reset устройства соответствует desktop reset.

### 10.5. Package render tests

Каждый опубликованный DSP package должен иметь:

- минимальный compile test;
- type/API test;
- reference program;
- numerical audio checks;
- reset test;
- block-invariance test;
- deterministic seed test, если есть random;
- documented tolerance;
- resource/capability expectations;
- regression fixtures прошлых releases.

Нельзя считать пакет совместимым только потому, что он компилируется.

## 11. Правила для код-агента

### 11.1. Этот документ не расширяет полномочия

Действуют правила `SPEC.md`: по умолчанию агент пишет тесты и разрешённый бойлерплейт. Наличие здесь будущего manifest, CLI, module syntax или registry model **не является разрешением реализовать их**.

### 11.2. Разрешённые ранние задачи

По отдельному запросу агент может:

- добавить tests для SourceMap/SourceSpan;
- добавить deterministic diagnostics/codegen tests;
- написать fixtures с несколькими synthetic sources;
- проверить, что compiler stages не вызывают filesystem;
- подготовить будущий module/package test matrix в виде документации;
- реализовать test helpers и generators;
- провести read-only review на глобальное состояние и скрытый I/O;
- добавить regression tests для найденных нарушений.

### 11.3. Запрещено без отдельного явного этапа

- добавлять `use`/`import`/`export` в parser;
- создавать `linen.toml` и manifest parser;
- добавлять `linen add/install/publish`;
- реализовывать registry, HTTP client или global cache;
- добавлять native dynamic loading;
- вводить stable public bytecode ABI;
- делать общий plugin framework;
- разделять проект на workspace ради будущих пакетов;
- переписывать runtime на `no_std`;
- добавлять semver resolver;
- вводить network/crypto dependencies;
- создавать traits `Ecosystem`, `Plugin`, `ModuleLoader`, `Target` без текущего потребителя.

### 11.4. Если ранняя подготовка мешает Beep

Приоритет:

1. корректная текущая семантика;
2. простой код;
3. тестируемость;
4. минимальные multi-source/determinism seams;
5. будущая экосистема.

Нельзя задерживать Beep из-за проектирования registry. Если подготовительная абстракция требует больше кода, чем текущая функциональность, это признак преждевременного решения.

### 11.5. Prompt для текущей подготовительной задачи

```text
Прочитай SPEC.md и документ о подготовке к экосистеме.
Текущий этап: [ЭТАП]. Модульную систему и пакеты реализовывать запрещено.

Задача: добавить тесты для [SourceMap / SourceSpan / deterministic diagnostics /
origin preservation / другой текущий инвариант].

Разрешено менять только: [TESTS/FIXTURES].
Production-код не менять. Не добавлять зависимости и будущие abstractions.
Используй synthetic in-memory sources, а не реальные пути.
Проверь, что старые single-file programs сохраняют поведение.
Если production-код не позволяет проверить инвариант, оставь minimal failing
regression test по согласованию и опиши необходимое ручное изменение.
```

## 12. Чек-лист для каждого нового языкового/DSP-изменения

Перед merge ответить на вопросы:

### Source и diagnostics

- [ ] Есть ли `SourceSpan`?
- [ ] Сохранится ли он после lowering?
- [ ] Можно ли показать ошибку внутри будущей зависимости?
- [ ] Не зависит ли diagnostics от absolute path?

### Names и packages

- [ ] Не используется ли строка как глобальная уникальная идентичность?
- [ ] Не появился ли скрытый глобальный namespace?
- [ ] Можно ли будет сделать определение приватным?
- [ ] Не зависит ли resolution от filesystem order?

### Determinism

- [ ] Нет ли системного времени, random state или network?
- [ ] Не влияет ли `HashMap` iteration на output?
- [ ] Повторяется ли результат в новом process?
- [ ] Random stream не зависит от NodeId/address?

### Capabilities и resources

- [ ] Известна ли runtime capability нового узла?
- [ ] Известны ли state и memory requirements?
- [ ] Можно ли статически ограничить ресурсы?
- [ ] Что произойдёт на restricted target?

### Runtime и продукты

- [ ] VM остаётся независимой от CLI/WAV/host?
- [ ] Audio steady-state остаётся без allocations/I/O?
- [ ] Можно ли отделить debug info без изменения звука?
- [ ] Можно ли проверить поведение headless?

### Compatibility

- [ ] Добавлены ли regression и compatibility tests?
- [ ] Отдельно ли рассмотрены API compatibility и render compatibility?
- [ ] Не изменился ли старый preset незаметно?
- [ ] Обновлены ли SPEC.md/ADR до кода при изменении контракта?

## 13. Минимальный практический объём по текущим этапам

Подготовка распределяется по реально существующим compiler stages и не превращается в отдельную инфраструктуру:

```text
Этап 1A, до type checker и Beep WAV:
1. SourceId + SourceFile + SourceMap + SourceSpan.
2. Diagnostics знают logical source name и различают synthetic sources.
3. Compiler stages получают in-memory source и не делают I/O.
4. Compile config/session принадлежат явно.
5. Повторная компиляция и порядок diagnostics детерминированы.
6. export/package/from/as зарезервированы без module/package grammar.
7. Ничего пакетного и сетевого не реализуется.

Этап 2:
8. Resolved names используют BindingId/FunctionId/BuiltinId.
9. Typed AST сохраняет SourceSpan; sine получает базовый BuiltinDescriptor.

Этап 3:
10. IR сохраняет NodeOrigin; debug origin можно отделить без изменения звука.
11. IR ops получают capability/resource classification в объёме текущих узлов.

Этапы 4+:
12. Function/call origins и stable random stream identity появляются только
    вместе с соответствующей функциональностью и ADR.
```

Точный объём и DoD ближайшей миграции находятся в `docs/development/stages/stage-01a-ecosystem-foundation.md`. Всё остальное в этом документе — будущие этапы и защита от неверных решений, а не текущий backlog.

## 14. ADR, которые понадобятся позднее

Текущее решение о `SourceMap`, `SourceSpan` и `CompileSession` уже оформлено как ADR 0012. Для следующих тем нужны отдельные ADR при наступлении соответствующих этапов:

1. Module identity and logical names.
2. Import/export syntax and visibility.
3. Module cycle policy.
4. Function instantiation across modules.
5. Package manifest format.
6. Source dependency resolution.
7. Lockfile and content addressing.
8. Language/VM/package compatibility versions.
9. Capability and resource descriptors.
10. Parameter identity and preset migration.
11. Standard library versioning.
12. Registry namespace and immutable releases.
13. Package archive security limits.
14. Device package format and verification.
15. Embedded VM compatibility.
16. Random stream identity across modules and optimizations.

## 15. Итоговое правило

**Сейчас строится язык и проверяемый runtime, а не экосистема. Но каждый compiler stage обязан работать с явным источником, явным контекстом, стабильными resolved IDs и детерминированным поведением.**

Этого достаточно, чтобы позднее последовательно добавить модули, локальные пакеты, lockfile, registry, инструменты и embedded packages, не превращая раннюю разработку в бесконечное проектирование инфраструктуры.

## Changelog

### 0.2.1 — 2026-11-16

- Этап 1A реализован и принят по `SPEC.md` 0.2.1.
- После ревью уточнена область `SourceId`: это map-local индекс, который используется только с исходной картой или её идентичной копией. Независимые карты могут выдавать равные ID; глобальный provenance и mutable ID generator не добавляются.
- Исторические требования версии 0.2 сохранены ниже без заднего переписывания.

### 0.2 — 2026-11-15

- Ближайшие архитектурные требования интегрированы в `SPEC.md` 0.2.0 и этапы 1A, 2 и 3.
- Добавлена спецификация корректирующего этапа 1A и ADR 0012; текущая однофайловая реализация не объявляется уже мигрированной.
- Принято решение зарезервировать `export`, `package`, `from`, `as` без добавления module/package syntax.
- Уточнены правила исправления завершённых этапов: исторические отчёты сохраняются, а миграция получает отдельный DoD и compatibility tests.

### 0.1 — 2026-11-11

- Зафиксирована будущая экосистема source libraries, instruments, presets и device packages.
- Разделены source packages и исполняемые device artifacts.
- Определены ранние seams: SourceMap, SourceSpan, CompileSession, resolved IDs, NodeOrigin и deterministic compilation.
- Добавлены capabilities, resource analysis, parameter metadata и compatibility rules.
- Описаны этапы от multi-file project до registry и embedded distribution.
- Зафиксированы тестовые требования и запрет преждевременной реализации пакетной инфраструктуры агентами.
