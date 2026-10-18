develop을 main에 머지할 때 릴리즈를 자동으로 하려고 해.

# 릴리즈
- develop을 main에 머지하면 자동으로 릴리즈함
- GitHub Actions에서 실행 (GitHub의 macOS 실행기에서 빌드, 서명, 공증)
  - 서명에 필요한 인증서와 공증 키는 저장소 Secrets에 등록
- GitHub Releases에 설치본과 제거본을 올림
- 태그는 Cargo.toml의 버전으로 만듦 (예: v0.1.1)
  - 같은 태그가 이미 있으면 실패 (버전을 올리지 않은 머지 방지)
- 릴리즈 노트는 지난 릴리즈 이후 spec/merged에 새로 들어온 명세 요청서 목록
