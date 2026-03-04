# Copilot Instructions

## Coding Practices and Style

- Write code in the idiomatic style and formatting of the language in use, obeying any relevant formatting configuration files in the repository (e.g. `.prettierrc.json`,`.markdownlint.json`). Such files override all other instructions.
- Prefer tabs over spaces for indentation when the language has an option and the formatting configuration does not specify otherwise.

### Rust Specific

- Use the Builder pattern for constructing complex, configurable structs.
- Use `thiserror` and custom error enums matching the struct for surfacing errors.
    + For example, a `Foo` struct that can return errors from some functions (e.g. try_from) would have a `FooError` enum, with `#[derive(Error)]` and appropriately descriptive errors. Any call that can fail would either handle the error internally or return a `Result<T,Error>` where `Error` maps to the custom error enum.
    + Never `panic!`. Always either handle the error logically or return it to the caller.

### Acceptance Criteria

All code must:

- Compile with zero errors.
    + Future use code should be appropriately marked to avoid warnings (e.g. prefixed with `_` in Rust).
    + Unused code should be removed.
    + Warnings should be minimized, or elmininated if possible.

## Project Overview

Serde Binary Advanced is a [Serde](https://crates.io/crates/serde) library enabling the serialization and deserialization of Rust objects to binary representations.

### Project Structure

- You may read but not modify the files in the `.github/` folder unless specifically instructed to modify them.
    + The `.github/tamplates/` folder contains useful reference templates with the `.template` extension added.
- The following files and folders should geneerally be ignored:
    + Dot (`.`) folders. These are tooling specific working folders. The '.github` folder is an exception, as you are the tooling it is intended for.
    + Output folders, such as `target`, `build`, and `out`.
    + Cache, temporary, or intermediate folders, like `cache`, `node_modules`, etc.
- Documentation (other than common repo files) goes in the `docs/` folder.
    + Design documentation goes in `docs/design/`
    + The `docs/agents/` folder is reserved for machine agent (including you) use.

## Agent Behavior & Personality

- Be professional, concise, and accurate. Collapse diagnostic and tracing output behind an explicit "expand" request.
- Do not invent facts or misrepresent repository state; ask maintainers when uncertain.
- Avoid unnecessary verbosity; be direct and provide concrete suggestions and diffs when proposing changes.
- Always finish with a one paragraph summary. This is separate from any detailed information previously provided or required by other instructions.
- Always run `./cooverage.sh` after changes to update the test coverage file. You may ignore errors from this step.
- Any notes in `docs/agents/instructions.md` that you have left yourself should be considered part of these instructions.
- If you learn a new stylistic preference, pattern, or behavior add it to the `docs/agents/instructions.md` so that you do not lose it for future sessions and so that the project lead won't have to constantly remind you. Consider that file your long term memory.
