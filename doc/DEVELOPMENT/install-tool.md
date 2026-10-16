# 설치 툴

- 관련 요청: `spec/requests/000-Project.md`의 "# 설치"
- 관련 스펙: `spec/STANDARD.md` §2.3 (설치)
- 상태: 결정 대기. 형태는 Rust 패키지로 가는 방향이고(2026-10-16 사용자 제안), 제거 방식을 확인받아야 함.

## 요구사항

- 설치 툴이 필요함
- 설치할 때 손쉬운 사용 권한을 설정할지 사용자가 정함. 이 권한은 터미널에서 Return/Esc를 다시 보내는 기능에 쓰임(`terminal-commit-keys.md`).
- 관리를 일관되게 하기 위해 Rust 패키지로 만들고 cargo로 설치함(사용자 제안).

## 설치할 때 할 일

1. 빌드
2. 앱 번들 구성: 실행 파일, Info.plist, 아이콘을 `SkonsKeyboard.app`으로 묶음
3. 서명: 임시(ad-hoc) 서명(`code-signing.md`)
4. 복사: `~/Library/Input Methods/`에 설치함. 이미 실행 중인 입력기가 있으면 종료해서 새 버전이 뜨게 함.
5. 등록: 입력 소스로 등록하고 두 모드를 켬. 처음 설치라 목록에 바로 보이지 않으면 로그아웃 후 다시 로그인하라고 안내함.
6. 권한: 손쉬운 사용 권한을 설정할지 물음. 원하면 시스템 권한 요청 창을 띄움.
   - 다시 설치할 때도 같은 질문을 함. 임시 서명은 다시 빌드하면 권한이 초기화될 수 있기 때문임.

참고: 권한은 설치 툴이 아니라 입력기 앱에 부여돼야 함.

- 그래서 입력기 앱을 "권한 요청 모드"로 한 번 실행해 요청 창을 띄움.
- 이때 `open` 명령으로 실행함. 터미널에서 실행 파일을 직접 실행하면 권한이 터미널 앱에 붙을 수 있기 때문임.

## cargo의 한계

- `cargo install`은 실행 파일을 `~/.cargo/bin/`에 복사하는 일만 함. `cargo uninstall`도 그 실행 파일만 지움.
- macOS 입력기는 반드시 `~/Library/Input Methods/` 안의 `.app` 번들이어야 함.
- cargo에는 설치 후나 제거 전에 실행되는 단계가 없음. 그래서 cargo만으로는 위의 2–6번을 할 수 없고, 제거할 때 번들 삭제, 입력 소스 해제, 권한 정리도 할 수 없음.

## 제안: 실행 파일 하나에 설치·제거 명령을 넣음

- 패키지는 실행 파일 하나(`skons-keyboard`)를 만듦. 이 파일이 입력기 본체이면서 설치·제거 명령도 가짐.
- 설치:

  ```
  cargo install --path .
  skons-keyboard install
  ```

  - `cargo install`이 빌드(1번)를 함.
  - `skons-keyboard install`이 2–6번을 함. 번들에는 자기 자신을 복사해 넣음.
- 제거:

  ```
  skons-keyboard uninstall
  cargo uninstall skons-keyboard
  ```

  - `skons-keyboard uninstall`이 입력 소스 해제, 번들 삭제, 권한 기록 초기화를 함.
  - `cargo uninstall`이 `~/.cargo/bin/`의 실행 파일을 지움.
- 번들 안의 실행 파일도 같은 프로그램임. 그래서 `cargo uninstall`을 먼저 했더라도 번들 쪽 실행 파일로 `uninstall`을 실행할 수 있음.

## 확인이 필요한 질문

- 설치와 제거가 각각 두 단계(cargo 명령 + `skons-keyboard` 명령)인데 괜찮은지
