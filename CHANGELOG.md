# Changelog

## [0.3.0](https://github.com/decheverri123/claude-shift/compare/cshift-v0.2.0...cshift-v0.3.0) (2026-08-25)


### Features

* add manual release trigger with tag and prerelease inputs to workflow ([d45336a](https://github.com/decheverri123/claude-shift/commit/d45336a32fc39d1eac30701d95bdb4899c70d3c8))
* add workflow_dispatch trigger to all CI and release workflows ([7874fca](https://github.com/decheverri123/claude-shift/commit/7874fca0c90fe69bdd819984cdc2ccd6b5eb5a99))
* update configuration preset prompt to clarify selection behavior ([3ae77e6](https://github.com/decheverri123/claude-shift/commit/3ae77e64882346224da5e19e65c739d60db4fde4))


### Bug Fixes

* **installer:** resolve sha256sum single-asset check and consolidated release checksums ([81ee205](https://github.com/decheverri123/claude-shift/commit/81ee2059d161641eb5aa2ca1a3e0cbe4cd9eb91d))
* update binary compression path to correctly stage artifacts for release ([35f3c57](https://github.com/decheverri123/claude-shift/commit/35f3c5784d1a009ffbd8c8c23e8b484049082d93))

## [0.2.0](https://github.com/decheverri123/claude-shift/compare/cshift-v0.1.0...cshift-v0.2.0) (2026-08-24)

### Features

- add installation script, automated release workflows, and enhanced configuration management with open-config functionality ([f5f0ba6](https://github.com/decheverri123/claude-shift/commit/f5f0ba62198956a0dccf96d466d84bca00d62c19))
- add live configuration preview in wizard and improve local Ollama model verification ([2e53eea](https://github.com/decheverri123/claude-shift/commit/2e53eea41993f8461069bf2fde298bdf205c5f9a))
- initial release of claude-shift ([b6014a2](https://github.com/decheverri123/claude-shift/commit/b6014a299ce864262b6b9668f47fb22473da2da6))
- rebuild claude-shift as a single Rust binary ([8072d2d](https://github.com/decheverri123/claude-shift/commit/8072d2d63ebcfb136783a22bc2bee5203edd33ce))

### Bug Fixes

- wire release-please workflow to config and manifest files ([528873e](https://github.com/decheverri123/claude-shift/commit/528873e32ce7e10acec314618d2a61d7846fd02e))
