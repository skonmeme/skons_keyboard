# skon's Keyboard

macOS용 한영 입력기입니다. 한글은 세벌식 최종, 영어는 QWERTY이며 Shift+Space로 전환합니다.

- 지원 환경: macOS 26 이상, Apple Silicon
- 스펙: [spec/STANDARD.md](spec/STANDARD.md)

## 설치

1. `skonsKeyboard-<버전>.pkg`를 엽니다.
2. "설치 유형"에서 필요하면 **터미널 Return/Esc 보정 (손쉬운 사용 권한)**을 선택합니다.
   - Terminal과 Ghostty에서 한글 조합 중 누른 Return과 Esc가 한 번에 동작하게 합니다.
   - 설치가 끝나면 권한을 묻는 창이 뜹니다. 시스템 설정을 열어 skonsKeyboard를 켭니다.
3. 관리자 암호를 입력하면 `/Library/Input Methods/`에 설치됩니다.
4. 메뉴 막대의 입력 메뉴에서 **Sebeolsik Final**(한글)이나 **QWERTY**(영어)를 선택합니다.
   - 보이지 않으면 시스템 설정 > 키보드 > 텍스트 입력의 입력 소스 "편집…"에서 "+"를 누르고 한국어 아래의 **skon's Keyboard**를 추가합니다.
   - 처음 설치할 때는 로그아웃한 뒤 다시 로그인해야 할 수 있습니다.

## 사용법

| 키 | 동작 |
|---|---|
| Shift+Space | 한글(세벌식 최종)과 영어(QWERTY) 전환 |
| Backspace (조합 중) | 마지막으로 친 자모를 지움 |

![세벌식 최종 배열](spec/STANDARD/sebeolsik-final-layout.svg)

- 한글 모드에서 숫자는 Shift와 오른손 자리 글쇠로 칩니다. `H J K L ;`가 0–4, `Y U I O P`가 5–9입니다.
- 한글 모드에서는 `` ` `` `@` `#` `$` `^` `&` `_` `[` `]` `{` `}` `|`를 칠 수 없습니다. 영어 모드로 바꿔서 칩니다.
- 손쉬운 사용 권한이 없으면 Terminal과 Ghostty에서 한글 조합 중 누른 Return과 Esc를 한 번 더 눌러야 할 수 있습니다. macOS 기본 한글 입력기도 같습니다.
- 암호 입력란에서는 macOS가 영문 입력만 허용하므로 한글 모드를 쓸 수 없습니다.

## 제거

`skonsKeyboard-Uninstall-<버전>.pkg`를 엽니다. 입력 소스를 끄고, 입력기를 삭제하고, 손쉬운 사용 권한 기록을 지웁니다.

## 설치본 만들기 (개발자용)

필요한 것:

- Rust stable과 Xcode 명령줄 도구
- 키체인의 Developer ID Application 인증서와 Developer ID Installer 인증서
- 공증용 `notarytool` 키체인 프로필 `skon's Universe`

```sh
cargo test
cargo xtask package
```

`target/package/`에 서명하고 공증받은 설치본과 제거본이 만들어집니다. 동작 방식은 [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md)에 있습니다.

## 라이선스

BSD 3-Clause. [LICENSE](LICENSE)를 봅니다.
