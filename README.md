# loco_study

Rust + [Loco](https://loco.rs) 프레임워크 기반 웹 애플리케이션 학습 프로젝트입니다.
JWT 인증, Articles CRUD, 서버사이드 렌더링(Tera + HTMX)을 포함합니다.

## 기술 스택

| 항목 | 내용 |
|------|------|
| Language | Rust 2021 edition |
| Framework | Loco 0.16 |
| Database | PostgreSQL (SeaORM) |
| Template Engine | Tera |
| Frontend | HTMX + Tailwind CSS |
| Auth | JWT + Magic Link |

## 프로젝트 구조

```
src/
├── controllers/
│   ├── article.rs          # Articles REST API
│   ├── auth.rs             # 인증 API (register, login, magic-link 등)
│   ├── notes.rs            # Notes API (stub)
│   ├── user.rs             # User API (stub)
│   └── admin/
│       ├── home.rs         # 관리자 페이지 뷰 라우트
│       └── articles.rs     # 관리자 Articles 뷰 (SSR)
├── models/
│   ├── articles.rs         # Article 모델 (search, before_save 훅)
│   └── users.rs            # User 모델 (JWT, 비밀번호, 이메일 인증)
├── initializers/
│   └── view_engine.rs      # Tera 뷰 엔진 초기화
└── workers/
    └── downloader.rs       # 백그라운드 워커

assets/views/
├── articles/
│   ├── list.html           # 목록 (페이지네이션 포함)
│   ├── form_new.html       # 신규 작성 폼
│   └── form_edit.html      # 수정 폼
├── components/
│   └── article_table.html  # 테이블 + 페이지네이션 컴포넌트 (HTMX)
├── index.html / blank.html / calendar.html / forms.html / tables.html
migration/
├── m20220101_000001_users.rs
└── m20260521_020341_articles.rs
```

## API 엔드포인트

### 인증 (`/api/auth`)

| Method | Path | 설명 |
|--------|------|------|
| POST | `/api/auth/register` | 회원가입 (이메일 인증 메일 발송) |
| GET | `/api/auth/verify/:token` | 이메일 인증 |
| POST | `/api/auth/login` | 로그인 → JWT 반환 |
| POST | `/api/auth/forgot` | 비밀번호 찾기 |
| POST | `/api/auth/reset` | 비밀번호 재설정 |
| GET | `/api/auth/current` | 현재 로그인 유저 정보 (JWT 필요) |
| POST | `/api/auth/magic-link` | 매직링크 발송 |
| GET | `/api/auth/magic-link/:token` | 매직링크로 로그인 |
| POST | `/api/auth/resend-verification-mail` | 인증 메일 재발송 |

> Magic Link는 `@example.com`, `@gmail.com` 도메인만 허용

### Articles REST API (`/api/articles`)

| Method | Path | 설명 |
|--------|------|------|
| GET | `/api/articles` | 목록 조회 (검색 + 페이지네이션) |
| POST | `/api/articles` | 게시글 생성 |
| GET | `/api/articles/:id` | 단건 조회 |
| POST | `/api/articles/:id` | 게시글 수정 |
| DELETE | `/api/articles/:id` | 게시글 삭제 |

**목록 조회 쿼리 파라미터:**

| 파라미터 | 설명 |
|----------|------|
| `title` | 제목 검색 (부분 일치) |
| `content` | 내용 검색 (부분 일치) |
| `created_at_from` | 생성일 시작 (`YYYY-MM-DDTHH:MM`) |
| `created_at_to` | 생성일 끝 (`YYYY-MM-DDTHH:MM`) |
| `page` | 페이지 번호 |
| `page_size` | 페이지 크기 |

**응답 예시:**
```json
{
  "results": [
    { "id": 1, "title": "...", "content": "...", "created_at": "...", "updated_at": "..." }
  ],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total_pages": 5
  }
}
```

### 관리자 뷰 페이지 (SSR)

| Method | Path | 설명 |
|--------|------|------|
| GET | `/` | 대시보드 홈 |
| GET | `/articles` | Articles 목록 페이지 |
| GET | `/articles/new` | 게시글 작성 페이지 |
| GET | `/articles/:id/edit` | 게시글 수정 페이지 |
| GET | `/forms` | 폼 예제 페이지 |
| GET | `/tables` | 테이블 예제 페이지 |
| GET | `/calendar` | 캘린더 페이지 |

## 주요 기능

### Articles 페이지네이션
- 5페이지 단위로 페이지 범위를 묶어 표시
- HTMX로 Edit / Delete 버튼이 페이지 새로고침 없이 동작
- 서버사이드에서 `prev_page` / `next_page` / `page_range` 계산 후 템플릿에 주입

### 인증 흐름
1. **일반 로그인**: 회원가입 → 이메일 인증 → 로그인 → JWT 발급
2. **매직링크**: 이메일 입력 → 토큰 메일 발송 → 링크 클릭 → JWT 발급

### 백그라운드 워커
- `DownloadWorker`: `BackgroundAsync` 모드로 동작하는 비동기 워커 (확장 가능)

## 실행 방법

### 1. PostgreSQL 준비

```sh
docker run -d \
  --name myapp-db \
  -e POSTGRES_USER=loco \
  -e POSTGRES_PASSWORD=loco \
  -e POSTGRES_DB=myapp_development \
  -p 5432:5432 \
  postgres:15
```

또는 환경변수로 기존 DB 지정:

```sh
export DATABASE_URL=postgres://user:password@host:5432/dbname
```

### 2. 서버 실행

```sh
cargo loco start
```

서버가 올라오면 `http://localhost:5150` 으로 접근합니다.

### 3. DB 마이그레이션

기동 시 `auto_migrate: true` 설정으로 자동 실행됩니다. 수동 실행이 필요한 경우:

```sh
cargo loco db migrate
```

## 설정

[config/development.yaml](config/development.yaml) 에서 주요 설정을 확인할 수 있습니다.

| 항목 | 기본값 |
|------|--------|
| 서버 포트 | `5150` |
| DB URL | `postgres://loco:loco@localhost:5432/myapp_development` |
| JWT 만료 | 7일 (604800초) |
| 메일 SMTP | `localhost:1025` |
| Worker 모드 | `BackgroundAsync` |
