# 코드 서명 방식

- 관련 요청: `spec/requests/000-Project.md`의 "대상 환경: macOS 27 이상"
- 관련 스펙: `spec/STANDARD.md` §2.3 (설치)
- 상태: 결정 대기 (macOS 27에서 실측 필요)
- 조사 시점: 2026-10

## 배경

- Apple Silicon에서는 모든 실행 파일에 서명이 필요함. 개인용 빌드에는 보통 임시(ad-hoc) 서명(`codesign --sign -`)을 씀.
- macOS 26에서는 임시 서명만으로 입력기가 동작한다는 사례가 있음(구름 입력기 개발 빌드, qingjian).
- macOS 27.0.1에서는 임시 서명한 입력기가 등록되지 않는다는 보고가 하나 있음(msime). 다만 그 입력기는 다른 설정(hardened runtime, Sparkle 포함)도 함께 쓰고 있어서 원인이 서명인지는 확실하지 않음. 미검증임.
- 음성 입력을 만든다면 마이크 권한이 서명 신원에 묶임. 임시 서명은 빌드할 때마다 신원이 바뀌므로 권한이 매번 초기화될 수 있음.

## 선택지

| 선택지 | 필요한 것 | 비고 |
|---|---|---|
| A. 임시(ad-hoc) 서명 | 없음 | 가장 단순함. macOS 27에서 동작하는지 확인 필요 |
| B. Apple Development 인증서 | Xcode에 Apple ID 로그인 (무료 계정 가능) | 빌드해도 서명 신원이 유지됨. 마이크 권한도 유지됨 |
| C. Developer ID 인증서 | 유료 Apple Developer Program | 배포용이라 개인 사용에는 과함 |

## 추천

- A로 시작함. macOS 27에서 등록이 안 되거나 음성 입력을 만들게 되면 B로 바꿈.

## 확인이 필요한 질문

- B가 필요해지면 Xcode에 Apple ID로 로그인해서 개발 인증서를 만들어도 되는지

## 출처

- Apple Silicon 서명 요구사항: https://developer.apple.com/documentation/macos-release-notes/macos-big-sur-11_0_1-universal-apps-release-notes
- macOS 27 임시 서명 보고 (msime): https://github.com/metasequoiaime/msime
- 임시 서명과 권한 초기화: https://developer.apple.com/forums/thread/707177
