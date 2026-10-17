# 코드 서명 방식

- 관련 요청: `spec/requests/000-Project.md`의 "# 설치"
- 관련 스펙: `spec/STANDARD.md` §2.3 (설치)
- 상태: 결정됨(2026-10-17). Developer ID로 서명하고 공증을 받음. 아래 준비가 끝나야 설치본을 만들 수 있음. 스펙에 적용할 때 이 문서는 삭제함.

## 결정

- 앱과 설치본을 사용자의 Apple Developer Program 계정(Developer ID)으로 서명하고 Apple 공증(notarization)을 받음.
- 얻는 것:
  - 내려받은 설치본이 Gatekeeper 경고 없이 열림
  - 앱 서명 신원이 업데이트해도 유지되므로 손쉬운 사용 권한(앞으로는 마이크 권한도)이 유지됨

## 이 Mac의 준비 상태 (2026-10-17 확인)

| 항목 | 상태 |
|---|---|
| Xcode에 연결된 개발자 계정 | 있음. 유료 개인 팀(무료 팀 아님) |
| 그 계정의 Apple ID가 사용자가 말한 주소인지 | 확인 못 함. Xcode가 계정을 내부 ID로만 저장함 |
| 멤버십 만료일 | 확인 못 함. developer.apple.com 로그인 필요 |
| Developer ID Application 인증서 (앱 서명용) | 없음 |
| Developer ID Installer 인증서 (설치본 서명용) | 없음 |
| 공증용 자격 증명 (`notarytool` 키체인 프로필) | 확인 안 함. 설치본을 만들 때 설정 |
| 도구 (`notarytool`, `productbuild`) | 있음 |

- 지금 있는 Apple Development 인증서는 개발용이라 배포용 서명에 쓸 수 없음.

## 준비할 일 (사용자가 직접)

1. Xcode > Settings > Accounts에서 계정이 맞는지 확인함.
2. 같은 화면의 "Manage Certificates"에서 Developer ID Application과 Developer ID Installer 인증서를 만듦. Developer ID 인증서는 계정 소유자만 만들 수 있으며, 개인 팀이면 본인이 소유자임.
3. 설치본을 만들 때 `xcrun notarytool store-credentials`로 공증용 자격 증명을 키체인에 저장함. 이때 앱 암호를 입력해야 하므로 직접 실행해야 함.

## 설치본을 만들 때 지킬 점

- 앱: Developer ID Application으로 서명하고 hardened runtime과 타임스탬프를 켬. 공증의 조건임.
- 설치본: Developer ID Installer로 서명함.
- 설치본을 공증받은 뒤 공증 기록을 설치본에 붙임(staple). 인터넷이 없어도 Gatekeeper가 확인할 수 있게 하기 위함.

## 출처

- 배포 전 공증: https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- Developer ID: https://developer.apple.com/developer-id/
- 임시 서명과 권한 초기화: https://developer.apple.com/forums/thread/707177
