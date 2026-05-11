# RustRiff
## From Rust to Rock in miliseconds!
**RustRiff** is a desktop guitar amplifier built with rust (**React + Typescript **, **Rust + Tauri**).
RustRiff models core amp controls (gain, tone stack, channel flow), an effect chain, and cabinet 
impulse-response (IR).

<p align="center">
  <a href="https://zacharyvds-is.github.io/Guitar-Amplifier/"><img alt="Docs" src="https://img.shields.io/badge/Docs-GitHub%20Pages-2563EB?style=for-the-badge"></a>
  <a href="https://zacharyvds-is.github.io/Guitar-Amplifier/frontend/index.html"><img alt="Frontend API" src="https://img.shields.io/badge/API-Frontend%20TypeDoc-0EA5E9?style=for-the-badge"></a>
  <a href="https://zacharyvds-is.github.io/Guitar-Amplifier/backend/doc/rustriff_lib/index.html"><img alt="Backend API" src="https://img.shields.io/badge/API-Backend%20Rustdoc-7C3AED?style=for-the-badge"></a>
  <a href="https://github.com/ZacharyVds-IS/Guitar-Amplifier"><img alt="Repository" src="https://img.shields.io/badge/Repository-GitHub-111827?style=for-the-badge"></a>
</p>

## Get Started

### Prerequisites

- Node.js 24+
- npm 10+
- Rust stable toolchain
- Tauri system dependencies for your OS

### Running locally

```powershell
npm install
npm run tauri dev
```

### Building for production
```powershell
npm install
npm run tauri build
```

### Work on documentation
RustRiff docs is a combination of custom written markdown and auto generated api references.
```powershell
//Development docs run
npm run docs:dev

//Building the combined documentation
npm run docs:build
```

## How to Contribute

1. Fork the repository and create a feature branch.
2. Keep changes scoped and aligned with the project architecture (`src`, `src-tauri/src/commands`, `services`, `domain`, `infrastructure`).
3. Add or update tests for success and failure paths when behavior changes.
4. Run checks before opening a PR.
5. Open a pull request with a short problem statement, solution summary, and testing notes.

## License
See `LICENSE.md`.
