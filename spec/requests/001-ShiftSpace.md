Ghostty에서 한영 전환(Shift+Space)을 하면 공백이 같이 들어가는 문제를 고치려고 해.

# 현상
- Ghostty에서 조합 중인 글자가 없을 때 Shift+Space를 누르면, 모드는 바뀌지만 공백도 함께 입력됨
- 다음 앱에서는 이런 현상이 없음: Zed, Spotlight, Firefox, Safari, 기본 터미널, Slack, Apple Mail, Apple Messages

# 원인
- Ghostty는 입력기가 키를 소비했다는 응답을 보지 않고, 키 처리 전후의 상태로 판단함
  - 조합 중인 글자가 있었는지
  - 입력기가 글자를 넣었는지
  - 입력 소스가 바뀌었는지
- 조합 중인 글자가 없으면 입력 소스 변경만 남는데, 모드 전환이 Ghostty의 키 처리가 끝난 뒤에 반영되어 Space로 처리됨

# 해결
- Ghostty에서 조합 중인 글자 없이 Shift+Space를 누르면, 모드 전환과 함께 빈 문자열을 넣음
  - Ghostty는 입력기가 글자를 넣은 키를 입력기가 처리한 것으로 보고, 빈 문자열에 대해서는 아무것도 보내지 않음
- "바뀌는 것이 없으면 앱을 호출하지 않는다"(STANDARD §7.2 규칙 4)의 예외로 둠
- Ghostty에만 적용함

# 확인
- Ghostty에서 조합 중인 글자가 없을 때와 있을 때 각각 Shift+Space를 눌러도 셸에 공백이 들어가지 않음 (STANDARD §7.3 A13)
