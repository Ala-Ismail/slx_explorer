# 1. SLX Viewer

**Windows x64** · [Download latest release](https://github.com/Ala-Ismail/slx_explorer/releases/latest) · [Release notes and checksums](https://github.com/Ala-Ismail/slx_explorer/releases) · [GitHub Actions builds](https://github.com/Ala-Ismail/slx_explorer/actions/workflows/windows.yml)

The Windows executables are unsigned. [Verify the ZIP checksum and read about Windows warnings](#123-verify-the-windows-download) before extracting and running the desktop app. The ZIP also includes `slx_viewer.exe` for command-line summaries.

## 1.1 Overview

SLX Viewer is an independent, open-source Windows x64 app for inspecting saved Simulink `.slx` files offline, without a MATLAB or Simulink installation. Browse saved diagrams and supported Stateflow charts in read-only mode. The inspector displays stored expressions as text. [MIT License](LICENSE)

SLX Viewer is a clean-room implementation developed independently of MathWorks, the maker of MATLAB and Simulink.

### 1.1.1 What’s new in v0.2.0

- A charcoal interface with a neutral grey canvas, compact model hierarchy, and a new folded-ribbon logo.
- Model-wide block search by name or system path. Select a result to reveal the block and its stored properties.
- Port-aware wire alignment for rotated and mirrored blocks, with saved block positions and branch junctions preserved.
- A 1.8-second startup screen with copyright and license notices. **Buy us a coffee** is available there and in **Help > About…**.


## 1.2 Build on Windows

To compile from source, install Rust and Cargo for the `x86_64-pc-windows-msvc` target and the Microsoft C++ build tools with a Windows SDK. Run the commands below in PowerShell from the repository root.

```powershell
cargo build --release --locked
```

The desktop build produces `target\release\slx_explorer.exe`. You can copy the executable to another folder and launch it there. It uses the Windows OpenGL graphics driver and links the C runtime into the executable. Cargo uses the dependency versions in `Cargo.lock`. If those dependencies are already cached, you can build offline with `cargo build --release --locked --offline`.

If a running viewer has the release executable locked, use `cargo build --release --locked --target-dir target/stateflow` to build the binaries under `target\stateflow\release`.

### 1.2.1 Dependency notices

Dependency and font notices are embedded in the executable and shown under **Help > License…**. After updating `Cargo.lock`, regenerate them with `python tools/update_notices.py` before building. The generator uses cached dependency sources and the upstream license copies in `notices`.

### 1.2.2 Download a GitHub build

The **Windows build** workflow runs on pushes to `main`, pull requests, version tags (`v*`), and manual dispatch. It checks formatting, Clippy, and tests before building Windows x64 release executables with the locked dependencies.

Open the repository's **Actions > Windows build**, select a successful run, and download **slx_explorer-windows-x64** under **Artifacts**. Extract the outer artifact to get `slx_explorer-windows-x64.zip` and `SHA256SUMS.txt`, then verify the inner ZIP using section 1.2.3. Artifacts are retained for 14 days.

To publish a release, commit the new version in `Cargo.toml` and `Cargo.lock` on `main`. Create an annotated tag matching that version and push the branch and tag together. Substitute the version you are releasing for `X.Y.Z`.

```powershell
$tag = 'vX.Y.Z'
git tag -a $tag -m "SLX Explorer $tag"
git push --atomic origin main "refs/tags/$tag"
```

A successful tagged build creates a GitHub Release with the ZIP and SHA-256 checksum under **Releases > Assets**. The release job uses GitHub's automatic token with permission to write repository contents. The workflow does not perform native GUI or clean-machine acceptance testing.

### 1.2.3 Verify the Windows download

1. Under **Assets**, download the Windows x64 ZIP and `SHA256SUMS.txt` from the same release. New builds use `slx_explorer-windows-x64.zip`. [Latest GitHub release](https://github.com/Ala-Ismail/slx_explorer/releases/latest)
2. Open PowerShell in the download folder and calculate the ZIP's SHA-256. If your ZIP has a different filename, substitute it in the command. [Microsoft Get-FileHash documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/get-filehash?view=powershell-5.1)

   ```powershell
   Get-FileHash -LiteralPath .\slx_explorer-windows-x64.zip -Algorithm SHA256
   ```

3. Compare the `Hash` result with the hash beside your ZIP's filename in `SHA256SUMS.txt`. Letter case can differ. If the hashes differ, stop and download both files again from the same release.
4. Once the hashes match, extract the ZIP and run `slx_explorer.exe`. Older releases use the filename `slx_viewer_desktop.exe`.

A matching checksum confirms that the ZIP matches the published digest. The desktop and command-line executables are unsigned. Microsoft Defender SmartScreen may show **Windows protected your PC** and list the publisher as **Unknown publisher**. Smart App Control or your organization's Windows policy may block the app. [Microsoft SmartScreen guidance](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation)

## 1.3 Open a model

Double-click the desktop executable. The startup screen shows the SLX Viewer logo, open-source MIT and copyright notices, and **Buy us a coffee** for 1.8 seconds before revealing the viewer. Choose **File > Open…** or press **Ctrl+O**, then select a local `.slx` file. To try the viewer, select `tests\fixtures\dc_motor_speed_control.slx` from the repository. You can also drop a model on the window, or provide a path when launching the app. Launch arguments begin loading during the startup screen.

```powershell
.\target\release\slx_explorer.exe
.\target\release\slx_explorer.exe "C:\Models\My Model.slx"
```

Use **Fit**, **−**, the zoom percentage, and **+** at the bottom right to control the current diagram. Clicking the percentage resets zoom to 100%. **Ctrl+wheel** zooms around the pointer. Long status messages truncate to keep these controls visible; hover over the message to read it in full.

Pan with the middle mouse button or by dragging empty canvas space. Click a block to inspect its saved properties. Open a subsystem from the hierarchy or by double-clicking the block. The **↑** toolbar button, **Esc**, or **Alt+Up** opens the parent. The **←** button or **Alt+Left** returns to the previously viewed system. Hover over either icon for its name and shortcut.

Click a signal line or branch to highlight its entire net. A diamond marks the source and circles mark the destination ports. The **Properties** panel lists the connected ports and flags unresolved endpoints. Click a port in the list to center it in the diagram at the current zoom. Click empty canvas to clear the selection.

Press **Ctrl+K** to search block names or system paths. Click a result or press **Enter** for the first match. The viewer selects the block in its diagram and shows its properties.

### 1.3.1 Command-line summary

Use `slx_viewer.exe` to print a summary in the terminal. Pass one local SLX path and quote paths containing spaces.

```powershell
.\target\release\slx_viewer.exe tests/fixtures/dc_motor_speed_control.slx
```

The fixture produces this summary.

```text
dc_motor_speed_control
Saved release: R2025b
Systems: 1
  dc_motor_speed_control: 25 blocks, 22 signals, 0 annotations [simulink/systems/system_root.xml]
```

The CLI prints recoverable diagnostics to standard error and still returns the parsed summary. A fatal load error prints an error message and returns a nonzero exit code. From a Rust program, call `slx_viewer::load(path, &slx_viewer::Limits::default())` to access the parsed model and diagnostics.

When a model contains supported Stateflow charts, the CLI also lists their block paths, chart IDs, and object counts. Rust callers can use `Block::chart` to index `Model::charts`.

### 1.3.2 View a Stateflow chart

Open a supported chart by double-clicking its block, pressing Enter with the block selected, or choosing it in the hierarchy. Select chart objects to inspect stored properties. Esc returns to the owning system; Back follows viewing history.

### 1.3.3 Menus and settings

**File > Open Recent** lists the ten most recently opened models, with their paths. Successful opens update the list. Settings are stored in `%APPDATA%\slx_viewer\settings.json`, including the default dark theme preference. Set `SLX_VIEWER_SETTINGS_DIR` to use another settings directory.

**Help > About…** shows the version, UTC build timestamp, source revision when available, target, profile, and graphics backend. **Buy us a coffee** opens the project's PayPal support page. **Help > License…** shows the project license and bundled dependency and font notices offline. The project is licensed under the [MIT License](LICENSE). Bundled dependencies and model fixtures retain their own license terms.

Use **Alt+F** or **Alt+H**, arrow keys, and **Enter** to operate the menus. **Esc** closes the active submenu or dialog before navigating the model.

## 1.4 Supported layouts and limits

- The loader discovers the block diagram and release metadata through package relationships, with fallbacks to the conventional block diagram and release metadata part names. Systems can be embedded in the diagram or resolved through the owning part's system relationships.
- The model retains system-local block identifiers, direct stored properties, saved block bounds, orientation and mirroring flags, declared port counts, ports observed in connections, annotations, and signal branch offsets. The canvas aligns wire ends with the estimated port edges and preserves saved block positions and branch junctions. Wire-end detours around endpoint blocks remain approximate. Specialized block icons are pending.
- Missing child parts and unresolved endpoints produce diagnostics. External models, model workspaces, defaults, specialized block semantics, and nested parameter structures are outside the current parser coverage. Only the fixtures listed in the [fixture manifest](tests/fixtures/README.md) have been exercised.
- The loader opens the file read-only and processes selected ZIP parts in memory. Defaults cap the compressed file at 128 MiB, the indexed entry count at 4,096, declared and consumed expanded data at 128 MiB, a read part at 16 MiB, and XML at 96 levels and 500,000 elements. System hierarchy has a separate depth cap of 32. The ZIP dependency builds its index before the entry-count check and coalesces duplicate entry names, which limits what that check covers.

## 1.5 Verification

Run these checks from the repository root. Formatting and Clippy use the `rustfmt` and `clippy` Rust components.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo build --release --locked
```

Release candidate `0.2.0`, `x86_64-pc-windows-msvc`, Windows build 26300. Earlier native visual checks used `dc_motor_speed_control.slx` at 175% scaling.

| Check | Result | Evidence |
| --- | --- | --- |
| Rust formatting | Pass | `cargo fmt --all -- --check` |
| Clippy | Pass | All targets and features, warnings treated as errors |
| Automated tests | Pass | Eleven integration tests and twenty-one desktop tests across all targets and features, using the retained motor fixture and synthetic cases |
| Locked Windows x64 release build | Pass | Built the CLI and desktop executables under `target\x86_64-pc-windows-msvc\release` with the GitHub Actions build command |
| Local Windows ZIP smoke test | Pass | Verified the package contents and SHA-256, extracted both executables, and opened the saved motor fixture with each. The fixture hash was unchanged |
| Published v0.2.0 ZIP integrity and signing status | Pass | The GitHub release ZIP matched its published SHA-256 on Windows. `Get-AuthenticodeSignature` returned `NotSigned` for both extracted executables |
| SmartScreen and Smart App Control prompts | Not run | Download guidance describes Microsoft's documented behavior. Prompt appearance on a clean or managed Windows device was not tested |
| Native launch and file picker | Pass | On an isolated Windows desktop, File > Open selected a fixture through the native dialog. After restart, Open Recent reopened the saved path. Settings retained two unique paths and the dark preference |
| T02 parser content | Pass for listed fixtures | Independently inspected saved XML and authored synthetic expectations cover counts, identity, hierarchy, properties, port information, and connectivity |
| T03 wire alignment | Pass for tested cases | Geometry tests verified saved branch junctions and perpendicular wire attachment at rotated and mirrored ports. Native visual review of the updated routing was not run |
| T01, T03–T04, T08, T10, T12–T13, T15–T16, T18 | Not run in full | Native smoke and automated checks cover picker and launch paths, branch geometry, pointer-anchored zoom limits, parent and Back state, unavailable interiors, read-only file access, malformed packages, and stale or cancelled completion handling |
| T05–T07, T09, T14, T17 | Not run in full | Manual canvas interactions, full keyboard priority, process and network observation, and clean-machine distribution remain to be verified |
| T11 Search and inspect | Pass for tested cases | Egui keyboard test selected BackEMF from the saved motor fixture and displayed `p.Ke` literally. A separate test resolved duplicate block names by qualified system path |
| T18 Compact hierarchy | Pass for tested cases | Windows build 26300 at 175% scaling showed compact, aligned rows and a contrasting label highlight in the saved microgrid model. Selecting Chopper opened its diagram. Expanding and collapsing its parent preserved the current diagram. An egui regression test confirmed that the hierarchy scrolls near the panel's right edge even with short labels |
| T21 File and recent models | Pass for tested cases | Tests cover persistence, equivalent-path deduplication, the ten-file limit, multiple app instances, failed opens, cancellation, and menu Esc handling. Native restart and recent-file selection passed |
| T22 Help dialogs | Pass for tested cases | Native review and egui tests checked actual build metadata, explicit unavailable revision, offline license notices, and Esc handling |
| T23 Dark theme | Not run in full | Native review confirmed the updated Simulink theme at 1200×800 and 740×460 logical pixels at 175% Windows scaling. The updated Stateflow theme and full display-scaling matrix remain pending |
| T30 Startup screen | Pass for automated cases | Egui tests checked the description, license notice, copyright, timed transition to a loaded fixture, and PayPal action from the cup icon. Native visual review of the updated splash was not run |
| T31 Signal net selection | Pass | Scene and egui tests verified nearest-wire picking, independent crossings, nested branches, endpoint markers, unresolved endpoints, and selection through pan and zoom. A local v0.2.0 release build on Windows build 26300 at 175% scaling highlighted the same net from the trunk and a branch of `dc_motor_speed_control.slx`, centered a destination, and cleared the selection on empty canvas. The fixture's SHA-256 was unchanged |
| Shared model tabs and menu row | Pass for tested cases | Egui checked that overflow tabs can scroll into view while File and Help stay in place. Native review on Windows build 26300 at 175% scaling opened the motor fixture through File > Open Recent from the microgrid fixture, switched back to the first tab, and opened Help with both tabs visible |
| Display scaling and performance | Not run | Display scaling matrix and reference-machine measurements remain pending |

No failures remained in the final checks above. See [the design](slx_viewer_design.md#111-implementation-milestones) for the remaining milestones.

### 1.5.1 SLX used

The accepted saved SLX references are only `dc_motor_speed_control.slx` and models under `tests/fixtures/microgrid/` (the micro_grid collection). See the [fixture manifest](tests/fixtures/README.md) for hashes and verification status. Other local models are excluded from reference tests and compatibility evidence. Synthetic archives remain permitted for isolated tests. Microgrid parser and native acceptance checks have not run. Earlier evidence from removed fixtures does not establish current acceptance.
