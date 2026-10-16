# 터미널에서 조합을 끝내는 키가 무시되는 문제

- 관련 스펙: `spec/STANDARD.md` §7.4 L1
- 상태: 결정 대기 (입력기 구현 후 실측 필요)
- 조사 시점: 2026-10

## 현상

- Terminal과 Ghostty에서 한글을 조합하다가 Return이나 Esc를 누르면 글자는 확정되지만 키 자체는 무시됨. 그래서 한 번 더 눌러야 함.
- Ghostty는 Tab과 화살표 키에도 같은 문제가 있음. 현재 안정 버전인 1.3.1에서는 오히려 확정된 글자가 사라지는 경우가 있음.
- macOS 기본 한글 입력기에서도 똑같이 일어남. 즉, 터미널 쪽 동작임.

## 원인

- 우리 입력기는 기본 입력기와 같이 "글자를 확정한 뒤 키를 앱에 넘기는" 방식으로 동작함.
- 일부 터미널은 키를 처리하는 도중 입력기가 글자를 넣으면, 그 키를 입력기가 처리한 키로 보고 버림.
- InputMethodKit 범위 안에서는 이를 피할 방법이 확인되지 않음.

참고로 Space는 이 문제를 피하려고 입력기가 직접 넣도록 스펙에 정함(§4.4).

## 선택지

| 선택지 | 장점 | 단점 |
|---|---|---|
| A. 그대로 둠 | 단순함. 기본 입력기와 같음. 추가 권한 불필요 | 해당 키를 두 번 눌러야 함 |
| B. 확정 후 키 이벤트를 다시 보냄 (CGEvent) | 한 번에 동작할 가능성 있음 | 손쉬운 사용(Accessibility) 권한 필요. 스펙 §2.2와 충돌. 이벤트 순서 보장이 검증되지 않음 |
| C. 터미널에서만 제어문자를 직접 넣음 (예: Return → `\r`) | 추가 권한 불필요 | Ghostty 개발 버전은 제어문자를 걸러냄. 앱별 예외 처리가 필요함. 스펙 §7.2 규칙 5와 충돌 |
| D. 터미널 쪽에 수정 요청 | 근본적인 해결 | 우리가 통제할 수 없음. iTerm2는 자체적으로 이미 고침 |

## 추천

- A로 시작함. 입력기를 만든 뒤 Terminal과 Ghostty에서 실제로 어떤 키가 무시되는지 측정함.
- 불편이 크면 B를 별도 요청서로 검토함. 이 경우 권한 정책(§2.2)도 함께 바꿔야 함.

## 확인이 필요한 질문

- 터미널에서 Return을 두 번 눌러야 하는 불편을 감수할 수 있는지, 아니면 권한을 추가해서라도 고치고 싶은지

## 출처

- Ghostty 키 처리 코드: https://github.com/ghostty-org/ghostty/blob/main/macos/Sources/Ghostty/Surface%20View/SurfaceView_AppKit.swift
- Ghostty 관련 PR/이슈: https://github.com/ghostty-org/ghostty/pull/14272, https://github.com/ghostty-org/ghostty/issues/11461
- 기본 입력기의 같은 증상 (구름 입력기 이슈): https://github.com/gureum/gureum/issues/1
- iTerm2의 자체 수정: https://github.com/gnachman/iTerm2/commit/52fdf26
