# 1. SLX Explorer Website Design

## 1.1 Purpose

Build a public landing page for SLX Explorer. It should explain what the desktop app does, let Windows users find the right download, and give developers a direct route to the source and issue tracker. The website introduces the app and distributes it through GitHub Releases. Viewing models happens locally in the desktop app.

The primary audience is engineers and reviewers who receive a saved Simulink model and need to inspect its structure, connections, or saved properties. The secondary audience is developers evaluating or contributing to the project.

## 1.2 Page structure and copy

Use a single page with anchor navigation for **Features**, **Download**, and **Open Source**. Put **Buy us coffee** in the header, near the download, and in the footer. Use **Download for Windows x64** as the primary action and **View source on GitHub** as the hero’s secondary action.

### 1.2.1 Introduction

- **Heading** — SLX Explorer
- **Description** — An open-source Windows app for inspecting saved Simulink models.
- **Supporting line** — Browse diagrams, follow signals, and read stored block properties offline. Released under the MIT License.
- **Visual** — Show a three-image slideshow covering an automotive engine blueprint with a mock SLX control diagram, a realistic wind turbine close-up with an integrated blueprint cutaway, and a realistic race car with a mock Stateflow-style diagram.

### 1.2.2 What you can inspect

| Feature | Visitor-facing explanation |
| --- | --- |
| Diagram and hierarchy | Pan and zoom saved block diagrams, follow signal routes and branches, and open available subsystems. |
| Block search and inspection | Search by block name or system path. Select a result to reveal the block and read its saved properties. |
| Supported Stateflow charts | Open supported saved charts and inspect their states, transitions, and stored text. |
| Read-only local viewing | The app opens models from the local filesystem and displays saved values and expressions as text. |

Keep compatibility details near the feature list. Coverage is fixture-based. Some specialized blocks, external references, and Stateflow constructs have limited support. Link to the repository’s supported layouts and limits before making any broader compatibility claims.

Follow the feature list with a [illustrative SLX Explorer preview](public/assets/pid-control-preview.webp) of an illustrative PID speed-control model. Show the three parallel paths, the motor feedback loop, and the selected proportional gain with the stored value `Kp`. Provide a full-size image link alongside the caption.

### 1.2.3 Download and get started

Label the package **Windows x64 · ZIP · Unsigned executables**. Link **Download for Windows x64** to the latest GitHub release, where the Assets section lists the ZIP and checksum. Provide a **Download SHA-256 checksum** link to its checksum file. GitHub Actions builds and publishes the ZIP and checksum. Explain near the download that Microsoft Defender SmartScreen may show **Windows protected your PC** and **Unknown publisher**, and that Smart App Control or managed Windows policy may block launch.

1. Download the Windows x64 ZIP and `SHA256SUMS.txt` from the same GitHub release. New builds use `slx_explorer-windows-x64.zip`.
2. Calculate the ZIP's SHA-256 in PowerShell and compare it with the published digest for the filename shown in the release. Tell users to stop and download both files again if the hashes differ.
3. After a match, extract the ZIP and open `slx_explorer.exe`. Older releases use the filename `slx_viewer_desktop.exe`.
4. Choose **File > Open…** or drop a local `.slx` file into the window.

Explain that a match confirms the ZIP's integrity against the published digest. Link the hero download note to the verification instructions.

Mention `slx_viewer.exe` as the bundled command-line tool for printing a summary of a local model. Use the GitHub release page as the source of truth for any displayed version number, release date, or package size.

Place the coffee link below the download actions. Use the message “Like SLX Explorer? You can support its development. The download is free and support is optional.”

### 1.2.4 Open source and support

Identify the project as MIT-licensed and include **© 2026 Lambda Technologies Pty Ltd** in the footer. Explain that contributions and reproducible bug reports are welcome. Link to source, license, and issues. Ask issue reporters for the app version, Windows version, and a redistributable example when available.

Use **Buy us coffee** with a simple coffee-cup icon next to the visible label. Link to the same PayPal page as the desktop app. Keep the support action separate from the free download.

State in the footer that SLX Explorer is a clean-room implementation developed independently of MathWorks, the maker of MATLAB and Simulink. Link to the third-party notices.

### 1.2.5 Common questions

- **Do I need MATLAB or Simulink?** The Windows desktop viewer runs offline without them.
- **Can I edit or simulate a model?** The app provides read-only inspection of saved diagrams and properties.
- **Where do I check compatibility?** Use the repository’s supported layouts and fixture manifest. The viewer reports unsupported content and unresolved references where it can identify them.
- **Where are my files opened?** The Windows app reads local files. The website offers information and download links.

## 1.3 Visual and accessibility direction

Use the desktop app’s folded orange ribbon mark, charcoal surfaces, a neutral grey diagram area, and restrained blue for focus and interactive highlights. Use the orange accent for the main download action. On desktop, place the hero copy beside the slideshow. On narrow screens, stack them with the download action above the first image. Label the PID image as an illustrative preview. Use unedited app captures for step-by-step walkthroughs.

Use a readable sans-serif stack, 16–18 px body text, and consistent spacing across sections. Give buttons enough room for touch and keyboard use. Provide descriptive image alt text, visible focus outlines, a single page heading, and sufficient text contrast. With JavaScript disabled, keep the first image and all page links available.

### 1.3.1 Hero slideshow

| Order | Asset | Content |
| --- | --- | --- |
| 1 | `/assets/hero-engine.webp` | Automotive engine blueprint and mock SLX control diagram |
| 2 | `/assets/hero-wind.webp` | Realistic wind turbine close-up with a blueprint cutaway of the nacelle |
| 3 | `/assets/hero-race.webp` | Race car on a circuit with a mock Stateflow-style Brake, Turn In, and Accelerate loop |

Use a consistent 3:2 crop and label the example diagrams as mock models. Advance every 6.5 seconds and offer Pause, Previous, and Next buttons. With reduced motion enabled, start paused. Pause while the pointer is over the slide area and stop automatic playback when visitors navigate the slides manually.

## 1.4 Link destinations

| Label | Destination |
| --- | --- |
| Download for Windows x64 | [Latest GitHub release assets](https://github.com/Ala-Ismail/slx_explorer/releases/latest) |
| Download SHA-256 checksum | [Latest checksum file](https://github.com/Ala-Ismail/slx_explorer/releases/latest/download/SHA256SUMS.txt) |
| All releases | [GitHub Releases](https://github.com/Ala-Ismail/slx_explorer/releases) |
| View source on GitHub | [SLX Explorer repository](https://github.com/Ala-Ismail/slx_explorer) |
| License | [MIT License](https://github.com/Ala-Ismail/slx_explorer/blob/main/LICENSE) |
| Compatibility | [Supported layouts and limits](https://github.com/Ala-Ismail/slx_explorer#14-supported-layouts-and-limits) |
| Report an issue | [GitHub Issues](https://github.com/Ala-Ismail/slx_explorer/issues) |
| Buy us coffee | [PayPal support](https://www.paypal.com/paypalme/12dazapp) |

## 1.5 Vercel deployment

The site is static HTML and CSS. Its page, stylesheet, favicon, and visual assets are in `website/public/`. The design document and `vercel.json` sit beside that directory, outside the published output.

| Vercel setting | Value |
| --- | --- |
| Git repository | `Ala-Ismail/slx_explorer` |
| Root Directory | `website` |
| Framework Preset | `Other` |
| Build Command | Leave empty and skip the build step |
| Output Directory | `public` |
| Production branch | `main` |

Test the site on a Vercel preview deployment before promoting it. The Open Graph image uses the full HTTPS address on `slxviewer-website.vercel.app`. Verify the image and link preview after deploying. Keep binaries and checksums in GitHub Releases. Use the stable release assets page so app releases and site deployments can proceed independently. [Vercel build settings](https://vercel.com/docs/builds/configure-a-build)

## 1.6 Acceptance

- The first screen makes the app’s purpose, Windows x64 support, and download action clear. The layout is usable at 360, 768, and 1440 px widths.
- The download link opens the latest release assets, where the Windows x64 ZIP and its SHA-256 file can be downloaded. The instructions match the filenames in the ZIP.
- Source, license, compatibility, issues, and PayPal links resolve to their intended destinations. Confirm the PayPal destination without submitting a payment.
- Verify keyboard navigation, visible focus, contrast, responsive image legibility, and page access without JavaScript. Check the slideshow’s wrapping order, pause control, and reduced-motion behavior.
- Vercel serves the home page and static assets from `website/public/`. Metadata includes the site title, description, favicon, and a share image.
