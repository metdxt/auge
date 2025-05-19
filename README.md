# Auge

> Observe... and reshape the visual world line.

**Auge** – a command-line apparatus for image manipulation, forged in the fires of Rust. Designed for seamless integration into the data streams of Unix-style pipelines, it can even project its results directly onto compatible observer interfaces (terminals supporting iTerm/Kitty protocols).

## Capabilities

*   **Reality Alteration Protocols:** A complete collection of visual transformation algorithms:
      - Grayscale: Convert to monochrome perception
      - Gaussian Blur: Soften edges with radial diffusion
      - Dotart: Render as stippled pointillism
      - Dynthres: Dynamic threshold binarization
      - Edge: Highlight perceptual boundaries
      - Invert: Negative reality inversion
      - Resize: Alter dimensional proportions
      - Sepia: Apply antique temporal patina
*   **Temporal Data Streams:** Reads from stdin and writes to stdout, enabling the chaining of alterations in sequence.
*   **Interface Perception:** Automatically detects the terminal's display capabilities (iTerm/Kitty), reverting to a blocky representation if the necessary protocols aren't supported.
*   **Format Spectrum:** Handles a wide range of input and output data formats.

### Known Data Formats

| Format     | Input (Decode)         | Output (Encode)         |
|------------|------------------------|-------------------------|
| AVIF       | ✗                      | ✓                      |
| BMP        | ✓                      | ✓                      |
| DDS        | ✓                      | ✗                      |
| Farbfeld   | ✓                      | ✓                      |
| GIF        | ✓                      | ✓                      |
| HDR        | ✓                      | ✓                      |
| ICO        | ✓                      | ✓                      |
| JPEG       | ✓                      | ✓                      |
| OpenEXR    | ✓                      | ✓                      |
| PNG        | ✓                      | ✓                      |
| PNM        | ✓                      | ✓                      |
| QOI        | ✓                      | ✓                      |
| TGA        | ✓                      | ✓                      |
| TIFF       | ✓                      | ✓                      |
| WebP       | ✓                      | ✓                      |

## Acquiring the Apparatus

Pre-compiled instances for Windows (x86_64) and Linux (x86_64, aarch64) await in the designated [release zone](https://github.com/metdxt/auge/releases/latest).

Alternatively, synchronize via the Cargo package manager:

```bash
# Initiate synchronization sequence
cargo install auge
# OR from git repo directly
# cargo install --git https://github.com/metdxt/auge
```

## Operating Instructions

Once synchronized, consult the internal knowledge base via `auge --help` and `auge <command> --help` to understand the available operations.

## Example Scenario: Altering a Visual Sequence

```bash
# Input image undergoes grayscale transformation,
# followed by Gaussian blur (strength 3),
# finally materializing as output.png.
auge -i input.png grayscale | auge g-blur -s 3 > output.png
```

> The image's world line has been shifted. Chain other protocols similarly to achieve desired realities.

## Transmission Protocol

This apparatus is distributed under the MIT license.
