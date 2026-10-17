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
| Developer ID Application 인증서 (앱 서명용) | 있음. 2026-10-17 발급, 2027-02-01 만료 |
| Developer ID Installer 인증서 (설치본 서명용) | 있음. 2026-10-17 발급, 2027-02-01 만료 |
| 공증용 자격 증명 | 있음. App Store Connect API 키를 키체인 프로필 `skon's Universe`로 등록함. 2026-10-17에 `notarytool history`로 인증되는 것을 확인함 |
| 도구 (`notarytool`, `productbuild`) | 있음 |

- 지금 있는 Apple Development 인증서는 개발용이라 배포용 서명에 쓸 수 없음.

## 인증서 만료

- 두 Developer ID 인증서는 오늘 발급됐지만 2027-02-01에 만료됨. 발급한 상위 인증 기관(Developer ID Certification Authority)이 그날 만료되기 때문으로 보임.
- Apple 안내에 따르면 만료 후에는 다음과 같음.
  - 서명한 앱은 계속 내려받고 설치하고 실행할 수 있음.
  - 만료된 Developer ID Installer 인증서로 서명한 설치본은 유효한 인증서로 다시 서명해야 설치할 수 있음.
- 따라서 2027-02-01 전에 두 인증서를 새로 만들어야 함. 그 뒤에 만드는 설치본은 새 인증서로 서명함.

## 준비할 일 (사용자가 직접)

1. 완료: Xcode에서 계정 확인
2. 완료: Developer ID Application과 Developer ID Installer 인증서 생성
3. 완료: App Store Connect API 키를 키체인 프로필(`skon's Universe`)로 등록. 공증할 때 `xcrun notarytool submit --keychain-profile "skon's Universe"`로 사용함.

## 설치본을 만들 때 지킬 점

- 앱: Developer ID Application으로 서명하고 hardened runtime과 타임스탬프를 켬. 공증의 조건임.
- 설치본: Developer ID Installer로 서명함.
- 설치본을 공증받은 뒤 공증 기록을 설치본에 붙임(staple). 인터넷이 없어도 Gatekeeper가 확인할 수 있게 하기 위함.

## 출처

- 배포 전 공증: https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- Developer ID: https://developer.apple.com/developer-id/
- 임시 서명과 권한 초기화: https://developer.apple.com/forums/thread/707177
