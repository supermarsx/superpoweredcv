
![SuperpoweredCV Banner](superpoweredcv-banner.png)

SuperpoweredCV is a comprehensive tool designed for red-teaming ATS (Applicant Tracking Systems) and AI resume parsers. It allows you to scrape LinkedIn profiles, generate PDF resumes with embedded prompt injections, and analyze how these systems interpret the data.

The project consists of three main components:
1.  **Core CLI**: A Rust-based command-line tool for analysis and PDF generation.
2.  **Browser Extension**: A Chrome/Firefox extension to scrape profile data.
## CLI Reference

The `superpoweredcv` CLI provides a suite of tools for generating, injecting, and analyzing resumes.

### Global Options
*   `-c, --config <FILE>`: Path to a configuration file (YAML, JSON, TOML).

### Commands

#### `generate`
Generate a PDF resume from a scraped JSON profile with optional injections.

```bash
superpoweredcv generate --profile <PROFILE_JSON> --output <OUTPUT_PDF> [OPTIONS]
```

**Arguments:**
*   `-p, --profile <FILE>`: Path to the profile JSON file (Required).
*   `-o, --output <FILE>`: Output PDF path (Required).
*   `--injection <TYPE>`: Type of injection to apply.
    *   Values: `None` (default), `VisibleMeta`, `LowVis`, `Offpage`, `TrackingPixel`, `CodeInjection`, `UnderlayText`, `StructuralFields`, `PaddingNoise`, `InlineJobAd`.
*   `--intensity <LEVEL>`: Intensity of the injection.
    *   Values: `Soft`, `Medium` (default), `Aggressive`.
*   `--position <POS>`: Position of the injection (for `VisibleMeta`).
    *   Values: `Header` (default), `Footer`.
*   `--phrases <PHRASE>...`: List of phrases to inject. Can be specified multiple times.
*   `--generation-type <TYPE>`: Strategy for generating injection content.
    *   Values: `Static` (default), `LlmControl`, `Pollution`, `AdTargeted`.
*   `--job-description <TEXT>`: Job description text (required for `AdTargeted` generation).

#### `inject`
Inject a payload into an existing PDF file.

```bash
superpoweredcv inject --input <INPUT_PDF> --output <OUTPUT_PDF> --type <TYPE> [OPTIONS]
```

**Arguments:**
*   `-i, --input <FILE>`: Path to the input PDF (Required).
*   `-o, --output <FILE>`: Path to the output PDF (Required).
*   `--type <TYPE>`: Type of injection (Required). See `generate` for values.
*   `--payload <TEXT>`: Specific payload content (e.g., URL for tracking pixel, code for XSS).
*   `--phrases <PHRASE>...`: List of phrases to inject.
*   `--generation-type <TYPE>`: Strategy for generating content.
*   `--job-description <TEXT>`: Job description text.

#### `analyze`
Run an analysis scenario to test how an ATS parses the resume.

```bash
superpoweredcv analyze --scenario <SCENARIO_FILE>
```

**Arguments:**
*   `-s, --scenario <FILE>`: Path to the scenario definition file.

#### `demo`
Run the built-in demo scenario to verify system functionality.

```bash
superpoweredcv demo
```

#### `preview`
Generate a preview PDF showing where injections would be placed.

```bash
superpoweredcv preview --output <FILE>
```

#### `validate`
Validate a configuration file.

```bash
superpoweredcv validate --config <FILE>
```

#### `docs`
Open the documentation in your default browser.

```bash
superpoweredcv docs
```

## GUI Mode
Running `superpoweredcv` without arguments launches the graphical user interface.

### GUI Features
*   **Brutalist Design**: High-contrast, efficient interface.
*   **Independent Windows**: Settings, Logs, and Preview open in separate, pinnable windows.
*   **Visual Preview**: Real-time visualization of injection placement.
*   **LaTeX Builder**: Visual builder for resume sections.

## Getting Started

### 1. Install the Browser Extension
The extension allows you to easily grab profile data from LinkedIn to use as a base for your experiments.

*   **Chrome**: Load the `extension/` folder as an unpacked extension in `chrome://extensions`.
*   **Firefox**: Load `extension/manifest-firefox.json` as a temporary add-on in `about:debugging`.

See [extension/README.md](extension/README.md) for detailed instructions.

### 2. Build the CLI
The core logic is written in Rust. You'll need a Rust toolchain installed.

**Important:** All cargo commands must be run from the `core` directory.

```bash
cd core
cargo build --release
```

## Usage

The CLI provides several commands to manage the workflow from data ingestion to report generation.

### Generate a PDF Resume
Convert a scraped JSON profile into a PDF. This is useful for creating a baseline resume before applying injections.

```bash
cd core
cargo run -- generate --file <path/to/profile.json> --output resume.pdf
```

### Run an Analysis Scenario
Execute a red-teaming scenario defined in a configuration file. This simulates how an ATS might parse the resume with various injections.

```bash
cd core
cargo run -- analyze --scenario <path/to/scenario.yaml>
```

### Run the Demo
Run a built-in demo scenario to see the tool in action.

```bash
cd core
cargo run -- demo
```

### Validate Configuration
Check if your configuration files are valid.

```bash
cd core
cargo run -- validate --config <path/to/config.yaml>
```

## Project Structure

- `core/`: Rust CLI and library.
    - `src/profile.rs`: Data structures for profiles.
    - `src/generator.rs`: PDF generation logic.
    - `src/red_team.rs`: Injection and scenario logic.
- `extension/`: Browser extension source code.
- `docs/`: Documentation and specifications.

## Development

To run the project in development mode:

```bash
cd core
cargo run -- help
```
