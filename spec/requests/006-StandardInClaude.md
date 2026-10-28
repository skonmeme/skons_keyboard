CLAUDE.md를 읽을 때, 유일한 스펙 문서인 STANDARD.md를 읽을 수 있도록 CLAUDE.md에 STANDARD.md의 역할 및 개정 방법 규정을 넣으려고 해.

# 사용자의 요청
## spec/STANDARD.md
- 유일무이한 스펙을 규정하는 문서. 이 문서 외의 그 어떤 문서도 스펙이 아님
- 다른 문서와 충돌이 나는 경우는 무조건 STANDARD.md가 맞음
- 혼동을 최대한 피하기 위해 영어로만 작성
- 이해를 돕기 위해 SVG 그림 추가. spec/STANDARD에 연계된 파일 배치

## STANDARD.md 개정 방법 및 관련 소스코드 구현, 추가 문서 업데이트
1. spec/requests에 개정에 쓰이는 명세 요청서를 생성
2. STANDARD.md 업데이트하기 직전에 명세 요청서를 자동으로 commit
3. 개정된 STANDARD.md 스펙 문서를 기반으로 연관된 소스 코드 구현
4. 연관된 추가 문서 업데이트 (eg. README.md)

## 명세 요청서에 갖춰야 하는 내용
1. 사용자의 요청
2. 현재 파악된 상태
3. 명세서 수정 방향
4. 논의사항
5. 기타 연관된 문서
6. 관련해서 별도의 claude 메모리 사용하지 말 것

# 현재 파악된 상태
- CLAUDE.md에는 일반적인 코딩 지침(영어)만 있고, STANDARD.md에 대한 내용은 없음
- STANDARD.md의 역할(유일한 스펙, 충돌하면 STANDARD.md가 맞음, 영어로만 작성, spec/STANDARD/에 보충 파일)은 STANDARD 머리말과 §9.1에 이미 있음
  - 그림은 본문에 넣은 Mermaid 7개와 SVG 파일 1개(spec/STANDARD/sebeolsik-final-layout.svg)
  - Mermaid는 하나에 6–12줄. SVG는 흰 배경에 좌표를 직접 적은 22KB 파일이고, 만드는 도구는 저장소에 없음
- 개정 방법은 STANDARD §9.2–§9.5에 있음. 요청과 비교하면
  - 2번(STANDARD.md 업데이트 직전에 명세 요청서 자동 commit)이 없음
  - 명세 요청서에 갖춰야 하는 내용이 없음 (§9.2에는 파일 이름 규칙만 있음)
  - 논의사항은 doc/DEVELOPMENT/에 파일별로 둠 (§9.5)
  - §9.1 문서 표에 CLAUDE.md가 없음
- 릴리즈 노트에는 명세 요청서의 첫 줄이 들어감 (§8.7)
- 다음 규칙은 STANDARD.md와 CLAUDE.md에 없고 Claude 메모리에만 있었음. 메모리는 지우고 여기에 옮김
  - 대화에서 정한 내용은 명세 요청서에 적음. STANDARD.md 수정과 구현은 사용자가 요청할 때만 함
  - 새 브랜치를 만들지 않음. 명세 요청서가 main/develop에 있거나 commit되지 않았으면 사용자에게 물음
  - 명세 요청서는 사용자의 말투(한글, 짧은 항목)로 쓰고 사용자가 쓴 문구를 살림

# 명세서 수정 방향
- 머리말과 §9.1의 STANDARD.md 역할: 그대로 두고, 그림 규칙만 논의사항 2에 따라 고침
- §9.1: 문서 표에 CLAUDE.md 추가 (Claude가 읽는 지침. STANDARD.md를 가리킴. 영어로만 작성)
- §9.2 명세 요청서
  - 첫 줄은 요청을 한 줄로 요약 (릴리즈 노트에 쓰임)
  - 이어서 사용자의 요청, 현재 파악된 상태, 명세서 수정 방향, 논의사항, 기타 연관된 문서
  - 결과적으로 STANDARD에 반영되는 내용만 적음. 대화에서 정한 내용도 여기에 적음
  - 사용자의 말투(한글, 짧은 항목)로 쓰고 사용자가 쓴 문구를 살림
  - 요청과 논의에 관한 내용은 Claude 메모리에 두지 않음
- §9.3: 새 브랜치를 만들지 않음. 명세 요청서가 main/develop에 있거나 commit되지 않았으면 사용자에게 물음
- §9.4: 사용자가 요청할 때만 적용함. 첫 단계로 명세 요청서 자동 commit 추가
- §9.5: STANDARD에 반영되지 않는 논의사항은 doc/DEVELOPMENT/에 둠
- CLAUDE.md: 영어로만 작성. STANDARD.md의 역할과 개정 방법(1–4)을 짧게 적고, 나머지 규칙은 STANDARD §9를 따르게 함. 매번 읽지 않아도 되는 내용은 넣지 않음

# 논의사항
1. 개정 방법 1–4에 없는 규칙(작업 브랜치 §9.3, spec/merged로 이동과 마무리 commit §9.4)은 그대로 둘지
   - 결정: 제안대로. 그대로 두고 CLAUDE.md에서 STANDARD §9를 가리킴
2. "SVG 그림 추가": Mermaid 그림을 SVG 파일로 바꾸는 것인지, SVG 그림도 쓸 수 있다는 것인지
   - 미정: SVG 파일과 Mermaid의 장단점을 비교한 뒤 정함
3. CLAUDE.md에 넣을 내용의 언어 (지금 CLAUDE.md는 영어)
   - 결정: CLAUDE.md도 영문으로만
4. Claude 메모리에 있던 규칙(현재 파악된 상태)을 STANDARD §9나 CLAUDE.md에 넣을지
   - 결정: 메모리에 있는 규칙은 STANDARD로. CLAUDE.md에는 매번 읽혀야 하는 내용이 아니면 제외
5. 논의사항을 명세 요청서와 doc/DEVELOPMENT/(§9.5) 중 어디에 둘지
   - 결정: 제안대로 명확히 분리. 요청서는 결과적으로 STANDARD에 반영되는 내용. 그 외 논의사항은 DEVELOPMENT

# 기타 연관된 문서
- CLAUDE.md
- spec/STANDARD.md 머리말, §8.7, §9
