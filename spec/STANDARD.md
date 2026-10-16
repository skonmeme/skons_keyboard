# Skon's Keyboard — Standard

This is the specification of Skon's Keyboard. It is the only normative
document of the project: when any other document or the source code conflicts
with it, this document is correct and the other must be fixed (§8.1).

## 1. Overview

### 1.1 Purpose

Skon's Keyboard is a Korean/English input method for macOS. It is built for
one user's needs, not as a general-purpose input method: it provides only the
features in this document and is designed to stay lightweight.

### 1.2 Scope

| Feature | Section |
|---|---|
| Switching between Korean and English with Shift+Space | §3 |
| Korean input with the Sebeolsik Final layout | §4 |
| English input with the QWERTY layout | §5 |
| Voice input for Korean and English speech | §6 (not yet specified) |
| Guaranteed operation in a fixed list of applications | §7 |

Out of scope: other Hangul layouts (2-set, 3-90, ...), Old Hangul, Hanja
conversion, auto-correction or prediction, a settings UI or configuration
files, and platforms other than macOS.

### 1.3 Conventions

The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are used as described
in RFC 2119.

| Term | Meaning |
|---|---|
| Client | The text view of an application that receives input. |
| Mode | Korean mode or English mode of the input method. |
| Composition | The syllable being typed in Korean mode. The client shows it as marked text (underlined, not yet final). |
| Commit | Send the composition to the client as final text, then clear it. Committing an empty composition does nothing. |
| Pass through | Return a key event unhandled, so the client processes it as if no input method were active. |
| Consume | Handle a key event so that the client does not receive it. |
| Jamo | A Hangul letter. In Korean mode each jamo is an initial consonant (choseong), a medial vowel (jungseong) or a final consonant (jongseong). |

## 2. Platform and Technology

### 2.1 Platform

The input method runs on macOS 26 or later, on Apple Silicon (arm64). Other
macOS versions and Intel Macs are not supported.

### 2.2 Implementation

- The product is an input method bundle built on InputMethodKit. Typing
  (§3–§5) MUST work without a helper process, a login item, or the
  Accessibility or Input Monitoring permission.
- The implementation language is Rust. Apple frameworks are called through
  the objc2 crates.
- Swift MAY be used only for an Apple API that cannot be called from Rust.
  Every such use MUST be listed here. Current Swift uses: none.

### 2.3 Identity and installation

| Item | Value |
|---|---|
| Display name | Skon's Keyboard |
| Bundle | `SkonsKeyboard.app` |
| Bundle identifier | `com.skon.inputmethod.SkonsKeyboard` |
| Connection name | `com.skon.inputmethod.SkonsKeyboard_Connection` |
| Korean mode | ID `com.skon.inputmethod.SkonsKeyboard.Korean`, name "Sebeolsik Final", menu bar icon "한" |
| English mode | ID `com.skon.inputmethod.SkonsKeyboard.English`, name "QWERTY", menu bar icon "A" |
| Install location | `~/Library/Input Methods/` (current user only) |

macOS launches an input method only if its bundle identifier contains
`.inputmethod.`, and expects the connection name to be the bundle identifier
followed by `_Connection`.

The input method is built from source and installed with a single command,
described in README.md.

### 2.4 Resource and privacy constraints

- While no input happens, the input method MUST NOT run timers, polling or
  background threads.
- Typing in Korean or English mode MUST NOT perform file or network I/O.
- The input method MUST NOT store, log or transmit typed text.

## 3. Modes and Switching

The input method has exactly two modes: Korean mode (§4) and English mode
(§5). Each mode is a separate input source of the input method: Korean mode
has the Korean script, and English mode has the Roman script (ASCII-capable).
The menu bar therefore always shows the current mode, and applications can
tell whether keys are being composed.

```mermaid
stateDiagram-v2
    direction LR
    Korean: Korean mode (Sebeolsik Final)
    English: English mode (QWERTY)
    Korean --> English: Shift+Space (commit first)
    English --> Korean: Shift+Space
```

- **Shift+Space** is a Space key press while Shift is the only one of Shift,
  Control, Option, Command and Function (fn/Globe) that is held. Caps Lock is
  ignored.
- On Shift+Space the input method commits the composition, switches to the
  other mode and consumes the event. The client never receives Shift+Space.
- Switching MUST change the input source selected in the system, not only an
  internal state. Some terminals decide whether a key was consumed by
  checking whether the input source changed.
- Auto-repeated Shift+Space events (from holding the keys down) MUST be
  consumed without switching again.
- When the system changes the mode (for example, the user selects it in the
  input menu), the input method commits the composition and follows the
  change. It MUST NOT request a mode change in response, because doing so can
  start a feedback loop in some applications.

## 4. Korean Mode

### 4.1 Layout

Korean mode uses the Sebeolsik Final layout, also known as 3-91 or the
Gong Byeong-u Final layout. Keys are identified by their physical position on
an ANSI keyboard and named by their US QWERTY legend without Shift.

![Sebeolsik Final layout](STANDARD/sebeolsik-final-layout.svg)

In the tables below, "initial ㄱ", "medial ㅏ" and "final ㄱ" are the Unicode
conjoining jamo of that role (initial U+1100–U+1112, medial U+1161–U+1175,
final U+11A8–U+11C2). A value in code format is a character that is inserted
as is.

**Number row**

| Key | Without Shift | With Shift |
|---|---|---|
| `` ` `` | `*` | `※` |
| `1` | final ㅎ | final ㄲ |
| `2` | final ㅆ | final ㄺ |
| `3` | final ㅂ | final ㅈ |
| `4` | medial ㅛ | final ㄿ |
| `5` | medial ㅠ | final ㄾ |
| `6` | medial ㅑ | `=` |
| `7` | medial ㅖ | `“` |
| `8` | medial ㅢ | `”` |
| `9` | medial ㅜ | `'` |
| `0` | initial ㅋ | `~` |
| `-` | `)` | `;` |
| `=` | `>` | `+` |

**Top row**

| Key | Without Shift | With Shift |
|---|---|---|
| `q` | final ㅅ | final ㅍ |
| `w` | final ㄹ | final ㅌ |
| `e` | medial ㅕ | final ㄵ |
| `r` | medial ㅐ | final ㅀ |
| `t` | medial ㅓ | final ㄽ |
| `y` | initial ㄹ | `5` |
| `u` | initial ㄷ | `6` |
| `i` | initial ㅁ | `7` |
| `o` | initial ㅊ | `8` |
| `p` | initial ㅍ | `9` |
| `[` | `(` | `%` |
| `]` | `<` | `/` |
| `\` | `:` | `\` |

**Home row**

| Key | Without Shift | With Shift |
|---|---|---|
| `a` | final ㅇ | final ㄷ |
| `s` | final ㄴ | final ㄶ |
| `d` | medial ㅣ | final ㄼ |
| `f` | medial ㅏ | final ㄻ |
| `g` | medial ㅡ | medial ㅒ |
| `h` | initial ㄴ | `0` |
| `j` | initial ㅇ | `1` |
| `k` | initial ㄱ | `2` |
| `l` | initial ㅈ | `3` |
| `;` | initial ㅂ | `4` |
| `'` | initial ㅌ | `·` |

**Bottom row**

| Key | Without Shift | With Shift |
|---|---|---|
| `z` | final ㅁ | final ㅊ |
| `x` | final ㄱ | final ㅄ |
| `c` | medial ㅔ | final ㅋ |
| `v` | medial ㅗ | final ㄳ |
| `b` | medial ㅜ | `?` |
| `n` | initial ㅅ | `-` |
| `m` | initial ㅎ | `"` |
| `,` | `,` | `,` |
| `.` | `.` | `.` |
| `/` | medial ㅗ | `!` |

Korean mode cannot produce `` ` `` `@` `#` `$` `^` `&` `_` `[` `]` `{` `}`
`|`. To type them, switch to English mode.

### 4.2 Composition

The composition is a syllable with three slots in a fixed order:
initial < medial < final. Each slot is empty or holds one jamo. The *last
slot* is the filled slot that comes latest in that order.

When a jamo key (slot S, jamo j) is pressed:

1. If the composition is empty, or S comes after the last slot, put j into
   slot S.
2. Otherwise, if S is the last slot and the combination table below has an
   entry for (current jamo of S, j), replace slot S with the combined jamo.
3. Otherwise, commit the composition, then start a new composition with j in
   slot S.

```mermaid
flowchart TD
    K["Jamo key: slot S, jamo j"] --> A{"Composition empty, or<br/>S after the last slot?"}
    A -- yes --> P["Put j into slot S"]
    A -- no --> B{"S is the last slot and<br/>(current, j) is in the combination table?"}
    B -- yes --> R["Replace slot S with<br/>the combined jamo"]
    B -- no --> N["Commit, then start a new<br/>composition with j in slot S"]
```

Combination table:

| Slot | Combinations |
|---|---|
| Initial | ㄱ+ㄱ=ㄲ, ㄷ+ㄷ=ㄸ, ㅂ+ㅂ=ㅃ, ㅅ+ㅅ=ㅆ, ㅈ+ㅈ=ㅉ |
| Medial | ㅗ+ㅏ=ㅘ, ㅗ+ㅐ=ㅙ, ㅗ+ㅣ=ㅚ, ㅜ+ㅓ=ㅝ, ㅜ+ㅔ=ㅞ, ㅜ+ㅣ=ㅟ, ㅡ+ㅣ=ㅢ |
| Final | ㄱ+ㄱ=ㄲ, ㄱ+ㅅ=ㄳ, ㄴ+ㅈ=ㄵ, ㄴ+ㅎ=ㄶ, ㄹ+ㄱ=ㄺ, ㄹ+ㅁ=ㄻ, ㄹ+ㅂ=ㄼ, ㄹ+ㅅ=ㄽ, ㄹ+ㅌ=ㄾ, ㄹ+ㅍ=ㄿ, ㄹ+ㅎ=ㅀ, ㅂ+ㅅ=ㅄ, ㅅ+ㅅ=ㅆ |

A medial vowel typed after a final consonant always starts a new syllable
(rule 3). Unlike 2-set layouts, the final consonant never moves to the next
syllable.

**Backspace** while composing undoes the most recent keystroke of the
composition: the composition returns to the state it had before that
keystroke. When it becomes empty, the composition ends. Committed text is
never decomposed.

Examples (keys are QWERTY legends; `⌫` is Backspace):

| Keys | Composition after each key | Result |
|---|---|---|
| `m f s k g w` | ㅎ → 하 → 한 → ㄱ (한 committed) → 그 → 글 | 한글 |
| `n c ; t w n d x` | ㅅ → 세 → ㅂ → 버 → 벌 → ㅅ → 시 → 식 | 세벌식 |
| `k k f` | ㄱ → ㄲ → 까 | 까 |
| `k v f` | ㄱ → 고 → 과 | 과 |
| `k f w x` | ㄱ → 가 → 갈 → 갉 | 갉 |
| `k f x f` | ㄱ → 가 → 각 → ㅏ (각 committed) | 각ㅏ |
| `k f x ⌫ ⌫` | ㄱ → 가 → 각 → 가 → ㄱ | ㄱ |

### 4.3 Text representation

The composition is shown as marked text and committed as the same string:

- If the initial and medial slots are filled, it is the precomposed syllable
  U+AC00 + (L × 21 + V) × 28 + T, where L = initial − U+1100,
  V = medial − U+1161, and T = final − U+11A7 (T = 0 when the final slot is
  empty).
- Otherwise, it is the Hangul Compatibility Jamo (U+3131–U+3163) of each
  filled slot, in slot order. For example, an initial ㄱ alone is U+3131, and
  a medial ㅏ followed by a final ㄱ is U+314F U+3131.

### 4.4 Key handling

The first row that matches a key down event applies.

| Key event | While composing | While not composing |
|---|---|---|
| Shift+Space | Commit, switch to English mode, consume | Switch to English mode, consume |
| Any key with Command, Control, Option or Function held | Commit, pass through | Pass through |
| Backspace | Undo the last keystroke (§4.2), consume | Pass through |
| Layout key that produces a jamo | Compose (§4.2), consume | Start a composition, consume |
| Layout key that produces a character | Commit, insert the character, consume | Insert the character, consume |
| Space | Commit, insert a space, consume | Pass through |
| Any other key (Return, Tab, Escape, arrows, keypad, ...) | Commit, pass through | Pass through |

- A *layout key* is one of the 47 keys in §4.1, pressed with or without Shift
  and with no other modifier. Caps Lock is ignored: only Shift selects the
  column of the table.
- The input method MUST also commit the composition when the system asks it
  to (for example, the user clicks elsewhere, focus moves to another text
  view, or another input source is selected).

```mermaid
flowchart TD
    E["Key down (Korean mode)"] --> S{"Shift+Space?"}
    S -- yes --> T["Commit, switch mode, consume"]
    S -- no --> M{"Command, Control, Option<br/>or Function held?"}
    M -- yes --> P["Commit, pass through"]
    M -- no --> L{"Layout key?"}
    L -- "jamo" --> C["Compose, consume"]
    L -- "character" --> I["Commit, insert character, consume"]
    L -- no --> B{"Composing, and the key is<br/>Backspace or Space?"}
    B -- "Backspace" --> U["Undo last keystroke, consume"]
    B -- "Space" --> SP["Commit, insert space, consume"]
    B -- no --> P
```

Space during a composition is inserted by the input method, not passed
through, because some terminals drop a passed-through key that ends a
composition (§7.4).

## 5. English Mode

- Every key event is passed through, except Shift+Space (§3).
- In both modes, passed-through key events are interpreted with the ABC
  keyboard layout (`com.apple.keylayout.ABC`). English mode therefore types
  QWERTY, and keyboard shortcuts in Korean mode use QWERTY positions.

## 6. Voice Input

Requested: speak and have the recognized text inserted, for both Korean and
English speech, without choosing the language by hand.

Status: not yet specified. Discussion: `doc/DEVELOPMENT/voice-input.md`.
Until this section specifies it, the input method has no voice input.

## 7. Application Compatibility

### 7.1 Guaranteed applications

| Category | Applications |
|---|---|
| Web browsers | Safari, Firefox, Google Chrome |
| Editor | Zed |
| Terminals | Terminal, Ghostty |
| Messengers | Slack, Messages |
| Email | Mail |

Each guaranteed application MUST pass every scenario in §7.3, in its latest
stable version at the time of verification. Other applications are supported
on a best-effort basis.

### 7.2 Client interaction rules

These rules apply to every client. They follow what works across the
guaranteed applications.

1. The composition is shown as marked text from its first jamo, with a
   single underline and the cursor at its end. Uncommitted jamo are never
   inserted as final text.
2. Text is always inserted, and marked text always set, at the client's
   current selection. The input method never asks the client to replace a
   particular range.
3. While handling a key event, the input method inserts all text before it
   returns whether the event is passed through.
4. Committing is idempotent: each composition is committed at most once, even
   when the system asks for a commit late, repeatedly, or in the middle of
   another call. When nothing changes, the input method makes no call to the
   client.
5. Control characters (U+0000–U+001F, U+007F) are never inserted as text.
6. The input method handles key down events only. Mouse events and modifier
   key changes are never consumed.

### 7.3 Acceptance scenarios

Keys are QWERTY legends. Each scenario starts in an empty text field. In
terminals, the limitations in §7.4 apply to A2, A5, A10 and A11.

| # | Steps | Expected result |
|---|---|---|
| A1 | Korean mode: type `m f s k g w`, then Space. | `한글 ` is inserted. While typing, the composition is shown inline as marked text. |
| A2 | Korean mode: type `m f s`, then Return. | `한` is committed exactly once, then Return takes effect exactly once (new line, or send in a messenger). No character is duplicated, lost, or left behind in the field. |
| A3 | Korean mode: type `k f s`, then Backspace three times. | The composition goes 간 → 가 → ㄱ → empty, and nothing is inserted. |
| A4 | Korean mode: type `m f s`, then click elsewhere in the field. Repeat, switching to another application and back instead of clicking. | `한` is committed exactly once each time. |
| A5 | Korean mode: type `m f s`, then Left Arrow. | `한` is committed, then the cursor moves one character left. |
| A6 | Korean mode: type `m f s`, Shift+Space, then `abc`. | `한abc` |
| A7 | English mode: type `abc`, Shift+Space, then `m f s`. | `abc한` |
| A8 | Korean mode: type `m f s`, then Command+A, Command+C, Command+V. | `한` is committed, then the shortcuts act on QWERTY positions (select all, copy, paste). |
| A9 | Korean mode: type Shift+`j`, Shift+`k`, Shift+`l`, `[`, `]`. | `123(<` |
| A10 | Korean mode: type `m f s`, then Escape. | `한` is committed, then the application receives Escape. |
| A11 | Terminals only. Korean mode: type `m f s`, then Control+C. | `한` is committed, then the shell receives Control+C. |
| A12 | English mode: hold down `a`. | Key repeat works as with the ABC layout. |
| A13 | Terminals only. With nothing composed, press Shift+Space twice. | Nothing is sent to the shell, and the mode is back where it started. |

### 7.4 Known limitations

| # | Applications | Limitation |
|---|---|---|
| L1 | Terminal, Ghostty | When a passed-through key ends a composition, the terminal may keep the committed text but drop the key, so the key must be pressed again. This is reported for Return and Escape in both terminals, and for Tab and arrow keys in Ghostty, where version 1.3.1 may instead lose the committed text. The macOS built-in Korean input method has the same problem. Discussion: `doc/DEVELOPMENT/terminal-commit-keys.md`. |
| L2 | All | In password fields, macOS allows only ASCII-capable input sources, so Korean mode is not available there. |

### 7.5 Verification

- The composition rules (§4.2–§4.4) MUST be covered by automated tests that
  run without a client. The tests MUST pass before every commit.
- The scenarios in §7.3 are verified by hand in every guaranteed application
  whenever a change affects key handling or text output.

## 8. Project Documents and Process

### 8.1 Documents

| Path | Role | Language |
|---|---|---|
| `spec/STANDARD.md` | The specification (this document). The only normative document. | English only |
| `spec/STANDARD/` | Supplementary files of the specification, such as figures. | English only |
| `spec/requests/` | Spec requests waiting to be applied. | Any |
| `spec/merged/` | Spec requests already applied. | Any |
| `README.md` | Installation and usage, kept short. | Korean |
| `doc/ARCHITECTURE.md` | Detailed description of how the app works, with figures and videos. | Korean |
| `doc/ARCHITECTURE/` | Supplementary files of `doc/ARCHITECTURE.md`. | Korean |
| `doc/DEVELOPMENT/` | Open discussion items, one file per item. | Korean |

If any document or source code conflicts with this specification, the
specification is correct and the other MUST be fixed.

### 8.2 Spec requests

- A spec request is a file `spec/requests/NNN-Title.md`, where `NNN` is a
  three-digit sequence number starting at `000` (for example,
  `000-Project.md`).
- This specification changes only by applying spec requests.

### 8.3 Work branches

- Requests are applied on a work branch named `spec/NNN-title` (for example,
  `spec/000-project`). No work is done directly on `main` or `develop`.

### 8.4 Applying a request

```mermaid
flowchart TD
    R["spec/requests/NNN-Title.md"] --> W["Work branch spec/NNN-title"]
    W --> S["Update spec/STANDARD.md"]
    S --> C["Create or modify source code<br/>to match the specification"]
    C --> D["Update README.md and<br/>doc/ARCHITECTURE.md if affected"]
    D --> M["Move the request to spec/merged/"]
    M --> Q{"Any problem?"}
    Q -- no --> G["Commit on the work branch"]
    Q -- yes --> X["Report it, do not commit"]
```

1. Update this specification to reflect the request.
2. Create or modify the source code so that it matches the specification.
3. Update README.md and `doc/ARCHITECTURE.md` when they are affected.
4. Move the request file from `spec/requests/` to `spec/merged/`.
5. If no problem occurred, commit on the work branch automatically. A problem
   is a failed build or test, or a question that needs the user's decision.

### 8.5 Discussion items

- A question that needs discussion during development is recorded as one
  file in `doc/DEVELOPMENT/`.
- When the question is resolved, the file is deleted. If the answer changes
  this specification, the change is applied through a spec request.
