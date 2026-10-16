# 터미널에서 조합을 끝내는 키가 무시되는 문제

- 관련 스펙: `spec/STANDARD.md` §7.4 L1, §2.2 (권한 정책)
- 상태: 결정됨 (2026-10-16). B를 선택하고 Esc도 포함함. 요청서 `spec/requests/000-Project.md`에 반영했음. 스펙에 적용할 때 이 문서는 삭제함.
- 조사 시점: 2026-10-16. Ghostty, 구름 입력기, ongeul, iTerm2는 소스 코드를 직접 확인했고, Terminal.app은 공개 보고만 확인함.

## 현상

- Terminal과 Ghostty에서 한글을 조합하다가 Return이나 Esc를 누르면 글자만 확정되고 키는 버려짐. 그래서 한 번 더 눌러야 함.
- macOS 기본 한글 입력기와 구름 입력기에서도 똑같이 일어남.
- Ghostty에서도 일어남. 버전별 동작은 다음과 같음.
  - 1.3.1 (현재 안정 버전): 확정된 글자만 보내고 Return과 Esc는 보내지 않음. 화살표나 Tab으로 조합을 끝내면 확정 글자가 사라지는 문제도 있었음(#11461). 이 문제는 개발 버전에서 고쳐짐.
  - 개발 버전(main): 확정 후 위·아래·오른쪽 화살표만 다시 보냄. 왼쪽 화살표는 수식키가 붙었을 때만 다시 보냄. Return, Esc, Tab은 여전히 버림.

## 원인

- 한글 입력기는 조합 중 Return을 받으면 글자를 확정(insertText)하고 키를 앱에 넘김. 기본 입력기, 구름 입력기, 우리 입력기 모두 같음.
- 키를 넘기면 AppKit은 앱에 `insertNewline:` 같은 명령을 전달함. Ghostty는 이 명령을 버리고, 같은 키 입력 중에 확정된 글자가 있으면 "입력기가 처리한 키"로 간주함.
- Ghostty 1.3.1 코드 주석(`src/input/key_encode.zig`): "Enter/escape don't encode the PC-style encoding because we want to encode committed text"
- 일본어 입력기는 Return으로 변환을 확정하고 키를 소비하므로 이 동작이 맞음. 한국어는 확정과 동시에 Return이 실행되기를 기대하므로 맞지 않음.

## Ghostty 쪽 입장

- 관리자(mitchellh)는 2024년 #1663에서 일본어 입력기 기준으로 보면 정상 동작이라고 답함. 다만 "다른 입력기에 영향을 주지 않고 고칠 수 있다면 받아들이겠다"고 함.
- 2026-09에 원인을 정확히 짚은 수정 PR #14272가 올라왔음. 내용 문제가 아니라 기여자 인증(vouch) 정책 때문에 자동으로 닫힘.

## 다른 프로그램의 해결 방법

| 프로그램 | 방법 | 비고 |
|---|---|---|
| 구름 입력기 (본선) | 해결하지 않음. 기본 입력기와 같음 | HACKING.md에 "Terminal.app은 리턴이 조합만 해제"라고 적혀 있음. 이슈 #1이 2014년부터 열려 있음 |
| 구름 입력기 포크 (zidell/gureum) | 조합 중 Return이 오면 확정하고 원래 Return은 삼킴. 그다음 CGEvent로 Return을 다시 보냄 | 손쉬운 사용(PostEvent) 권한 필요. 권한이 없으면 기존 동작. 수식키 없는 Return만 대상 |
| ongeul | 같은 방식. 다시 보내는 이벤트에 표식을 달아 입력기가 다시 처리하지 않게 함 | 손쉬운 사용 권한이 있을 때만 동작. 모든 앱에 적용 |
| iTerm2 | 터미널 쪽에서 고침. 입력기가 확정한 뒤 넘긴 `insertNewline:`과 `deleteBackward:`를 직접 처리함 | 이슈 #13030. Ghostty PR #14272도 같은 방식 |

## 선택지

| 선택지 | 장점 | 단점 |
|---|---|---|
| A. 그대로 둠 | 단순함. 추가 권한 불필요 | 터미널에서 Return과 Esc를 두 번 눌러야 함 |
| B. 조합 중 Return을 다시 보냄 (구름 포크, ongeul 방식) | 터미널에서도 한 번에 동작함. 다른 한글 입력기 두 곳이 쓰는 검증된 방식 | 손쉬운 사용 권한 필요. 이 권한은 범위가 넓음. 스펙 §2.2 변경 필요. 임시 서명 빌드는 다시 빌드할 때마다 권한이 초기화될 수 있음(`code-signing.md`). 키를 아주 빠르게 이어 치면 순서가 바뀔 여지가 이론상 있음 |
| C. 제어문자를 직접 넣음 (예: `\r`) | 추가 권한 불필요 | Ghostty 개발 버전이 걸러냄. 스펙 §7.2 규칙 5와 충돌. 제외 |
| D. Ghostty에 수정 제안 | 근본 해결. 기본 입력기 사용자에게도 도움 | 기여자 인증이 필요하고, 반영 시점을 통제할 수 없음. Terminal.app은 고칠 수 없음 |

## 추천

- B를 선택 기능으로 넣음: 손쉬운 사용 권한이 있으면 Return을 다시 보내고, 없으면 A처럼 동작함.
- 대상 키는 Return부터 시작함. Esc도 같은 방식으로 넣을 수 있음(vim 사용 시 유용).
- D는 B와 별개로 해볼 만함. 이슈나 토론으로 먼저 제안하는 방법이 있음.

## 스펙 반영 시 함께 고칠 점

- `spec/STANDARD.md` §4.4는 "조합 중 Space를 입력기가 직접 넣는다"고 하면서, 근거로 "Space를 넘기면 일부 터미널이 버린다"를 들고 있음.
- 코드를 확인해 보니 Ghostty는 넘긴 Space를 정상 처리함. 시스템이 Space를 `insertText(" ")`로 전달하고, Ghostty가 이를 확정 글자와 함께 보냄.
- 따라서 근거를 바로잡아야 함. 기본 입력기도 Space를 직접 넣으므로 동작 자체는 유지해도 됨.

## 함께 확인된 점: Shift+Space

- Ghostty는 키 처리 전후로 현재 입력 소스 ID(`TISCopyCurrentKeyboardInputSource`)를 비교함. 바뀌었으면 입력기가 키를 처리한 것으로 보고 터미널에 보내지 않음.
- 그래서 Shift+Space로 시스템 입력 모드를 바꾸면 공백이 새지 않음. 스펙 §3에 정한 방식과 같음.
- 단, 모드 변경이 Ghostty 쪽에 즉시 반영되는지는 실측이 필요함(스펙 §7.3 A13).

## 결정

- B를 선택 기능으로 넣음: 손쉬운 사용 권한이 있으면 조합 확정 후 원래 키는 삼키고 같은 키를 다시 보냄. 권한이 없으면 기존 동작(A)을 유지함.
- 대상 키는 Return과 Esc임.
- 권한을 설정할지는 설치할 때 정함(`install-tool.md`). 입력기는 사용 중에 권한 요청 창을 띄우지 않고 권한이 있는지만 확인함.

## 출처

- Ghostty 1.3.1 키 인코딩: https://github.com/ghostty-org/ghostty/blob/v1.3.1/src/input/key_encode.zig
- Ghostty 개발 버전 키 처리: https://github.com/ghostty-org/ghostty/blob/main/macos/Sources/Ghostty/Surface%20View/SurfaceView_AppKit.swift
- Ghostty 입력 소스 확인: https://github.com/ghostty-org/ghostty/blob/main/macos/Sources/Helpers/KeyboardLayout.swift
- Ghostty #1663 (Esc 두 번): https://github.com/ghostty-org/ghostty/issues/1663
- Ghostty #14272 (수정 PR): https://github.com/ghostty-org/ghostty/pull/14272
- Ghostty #11461 (화살표로 확정 시 글자 사라짐): https://github.com/ghostty-org/ghostty/issues/11461
- 구름 입력기 HACKING.md: https://github.com/gureum/gureum/blob/main/HACKING.md
- 구름 입력기 이슈 #1: https://github.com/gureum/gureum/issues/1
- 구름 입력기 포크 커밋: https://github.com/zidell/gureum/commit/6e03ee3
- ongeul: https://github.com/hiking90/ongeul/blob/main/OngeulApp/Sources/OngeulInputController.swift
- iTerm2 수정 커밋: https://github.com/gnachman/iTerm2/commit/52fdf26
