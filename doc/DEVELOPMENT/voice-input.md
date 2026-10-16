# 음성 입력 방식 결정

- 관련 요청: `spec/requests/000-Project.md`의 "4. AI 기능 - 음성인식 자동입력 (한글, 영어 상관없이)"
- 관련 스펙: `spec/STANDARD.md` §6 (아직 미정)
- 상태: 결정 대기
- 조사 시점: 2026-10. 조사는 macOS 26.7.1 / Apple M4 Pro에서 했고, 대상 OS는 macOS 26 이상임.

## 정해야 할 것

1. 직접 만들지, macOS 기본 받아쓰기로 충분한지
2. 인식 엔진
3. 언어 처리: 자동 감지인지, 현재 입력 모드(한글/영어)를 따르는지
4. 조작 방법: 시작/종료 키
5. 구조: 입력기 프로세스 안에서 처리할지, 별도 도우미 앱으로 분리할지

## 핵심 사실

- 애플 음성 API(SpeechAnalyzer, SFSpeechRecognizer)는 모두 한국어(ko-KR)를 지원함. 단, 한 번에 언어 하나만 쓰며 자동 언어 감지 공개 API는 없음.
- 한영 혼용 발화의 공개 측정 자료는 HiKE 벤치마크뿐임. 오픈 모델 중에는 Whisper-Large가 가장 좋고, 클라우드 GPT-4o-transcribe가 그보다 좋음.
- 모든 모델이 영어 단어를 한글로 받아 적는 경우가 흔함(예: "bus" → "버스").
- 애플 엔진의 한국어 및 한영 혼용 정확도는 공개 자료가 없음. 직접 비교해야 알 수 있음.

## 후보 비교

| 후보 | 한영 혼용 | 오프라인 | 입력기 부담 | Rust 연동 | 비고 |
|---|---|---|---|---|---|
| A. macOS 기본 받아쓰기 | 언어를 손으로 전환 | 가능 | 없음 (개발 불필요) | 해당 없음 | 우리 입력기와 함께 동작하는지 확인 필요 |
| B. SpeechAnalyzer (macOS 26) | 언어 하나 | 가능 | 가벼움. 모델이 시스템 프로세스에서 돎 | Swift 연결층 필요 | 최신 API, 빠름, 권한은 마이크만 필요 |
| C. SFSpeechRecognizer | 언어 하나 | 가능 (음성 자산 설치 시) | 가벼움 | 순수 Rust (`objc2-speech`) | 구형, B보다 정확도 낮음, 음성 인식 권한 추가 |
| D. Whisper large-v3-turbo (whisper.cpp) | 오픈 모델 중 최고 (발화마다 언어 자동 감지) | 가능 | 무거움. 디스크 0.6–1.6 GB, 메모리 약 1–2 GB | `whisper-rs` 또는 `transcribe-cpp` | 키를 뗀 뒤 약 0.5–0.8초 지연 (추정) |
| E. 클라우드 API (OpenAI 등) | 가장 좋음 | 불가 | 가벼움 | HTTP 호출 | 음성이 외부로 나감, API 키 저장, 사용료 발생 |

## 언어 처리

- 자동 감지가 되는 후보는 D와 E뿐임. Whisper는 발화 하나에 언어를 한 번 판단함.
- B와 C는 현재 입력 모드로 언어를 정하는 방법이 자연스러움(한글 모드는 ko-KR, 영어 모드는 en-US). "한영 상관없이"를 완전히 만족하지는 못함.
- 한국어 문장 속 영어 용어는 B의 `contextualStrings`(자주 쓰는 용어 목록)로 어느 정도 보완할 수 있음. 효과는 미검증.

## 조작 방법

- 입력기는 자신이 선택돼 있고 텍스트 입력란에 포커스가 있을 때만 키를 받음. 따라서 음성 입력도 그때만 시작할 수 있음.
- 방식 후보:
  - 누르는 동안 녹음 (push-to-talk)
  - 한 번 눌러 시작하고 다시 눌러 끝내기
- 키 후보: 오른쪽 Option이나 오른쪽 Command를 단독으로 길게 누르기. 다른 키와 함께 누르면 일반 단축키로 처리함.
- fn(Globe) 키는 macOS가 입력 소스 전환과 받아쓰기에 쓰므로 피함.

## 구조와 권한

- 입력기 프로세스가 직접 녹음하면 구조가 단순함. 다른 입력기(msime)가 입력기 프로세스 안에서 마이크 권한을 요청하고 녹음하는 사례가 있으므로 가능해 보임. 단, 우리 입력기에서는 미검증임.
- 별도 도우미 앱을 두면 권한과 메모리(특히 D의 모델)를 입력기와 분리할 수 있음. 대신 구조가 복잡해짐.
- 마이크 권한은 코드 서명 신원에 묶임. 임시(ad-hoc) 서명은 다시 빌드할 때마다 권한이 초기화될 수 있으므로, Apple Development 인증서로 서명하는 것이 좋음. 이 인증서는 무료 Apple ID로도 발급받을 수 있음.

## 추천

1. 키보드 기능을 먼저 완성하고, macOS 기본 받아쓰기(A)를 우리 입력기와 함께 써봄. 언어 수동 전환이 괜찮다면 음성 기능을 직접 만들지 않는 것이 가장 가벼움.
2. 직접 만든다면 본인 발화 20개 정도로 B와 D를 비교한 뒤 정함. 한국어 문장 속 영어 단어를 섞어서 비교함.
   - B가 충분하면 B를 씀. 가볍고, 모델 관리를 시스템이 함.
   - 영어 단어가 너무 자주 한글로 적히면 D를 씀. 대신 무게를 감수함.
3. 구조는 도우미 앱으로 분리하는 쪽을 기본으로 검토함. 키보드 입력 경로를 가볍게 유지하기 위함.

## 확인이 필요한 질문

- 오프라인 필수 여부: 클라우드(E)를 아예 제외해도 되는지
- 무게 허용 범위: 음성 기능을 위해 디스크 약 600 MB와 사용 중 메모리 1–2 GB를 감수할 수 있는지
- 언어 처리: 현재 입력 모드를 따르는 방식(B, C)도 괜찮은지
- 조작 방법: 누르는 동안 녹음과 토글 중 무엇이 좋은지, 어느 키를 쓸지

## 출처

- SpeechAnalyzer: https://developer.apple.com/documentation/speech/speechanalyzer, WWDC25 세션 277 (https://developer.apple.com/videos/play/wwdc2025/277/)
- SFSpeechRecognizer: https://developer.apple.com/documentation/speech/sfspeechrecognizer, `objc2-speech` 0.3.2 (https://crates.io/crates/objc2-speech)
- HiKE 벤치마크(한영 혼용): https://arxiv.org/abs/2509.24613
- whisper.cpp: https://github.com/ggml-org/whisper.cpp, `whisper-rs` 0.16.0 (https://crates.io/crates/whisper-rs)
- macOS 받아쓰기: https://support.apple.com/guide/mac-help/mh40584/26/mac/26
- 마이크 권한: https://developer.apple.com/documentation/bundleresources/requesting-authorization-for-media-capture-on-macos, https://developer.apple.com/forums/thread/707177
- 입력기 안에서 녹음하는 사례 (msime): https://github.com/metasequoiaime/msime
