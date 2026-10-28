CLAUDE.md를 읽을 때, 유일한 스펙 문서인 STANDARD.md를 읽을 수 있도록 CLAUDE.md에 STANDARD.md의 역할 및 개정 방법 규정을 넣으려고 해.

# spec/STANDARD.md
- 유일무이한 스펙을 규정하는 문서. 이 문서 외의 그 어떤 문서도 스펙이 아님
- 다른 문서와 충돌이 나는 경우는 무조건 STANDARD.md가 맞음
- 혼동을 최대한 피하기 위해 영어로만 작성
- 이해를 돕기 위해 SVG 그림 추가. spec/STANDARD에 연계된 파일 배치

# STANDARD.md 개정 방법 및 관련 소스코드 구현, 추가 문서 업데이트
1. spec/requests에 개정에 쓰이는 명세 요청서를 생성
2. STANDARD.md 업데이트하기 직전에 명세 요청서를 자동으로 commit
3. 개정된 STANDARD.md 스펙 문서를 기반으로 연관된 소스 코드 구현
4. 연관된 추가 문서 업데이트 (eg. README.md)
