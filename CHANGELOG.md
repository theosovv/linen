# Changelog

В файле фиксируются заметные изменения проекта. Изменения языковых и внешних контрактов сначала вносятся в `SPEC.md` и её changelog; здесь они дополнительно отмечаются с указанием совместимости.

## Unreleased

### Added

- Минимальный Rust package с library и binary, зафиксированным toolchain и `Cargo.lock`.
- Структурированные `Span`, `Diagnostic`, `DiagnosticLabel`, `Severity` и начальные коды ошибок из спецификации.
- Integration tests diagnostics и инфраструктурного CLI, doctest публичного API.
- Настройки rustfmt, clippy и GitHub Actions CI.
- Spec-driven workflow, шаблон задачи, документ этапа 0 и ADR 0001–0002.
- Правила для ИИ-агентов, README, CONTRIBUTING и базовые файлы GitHub-репозитория.
- Лицензия MIT.

### Compatibility

- Существующие языковые программы и аудиоконтракты отсутствуют; обратной совместимости на этапе 0 пока нет.
- `check`, `run` и `inspect` не объявлены работающими и не имеют placeholder-реализаций.
