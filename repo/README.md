# ScholarVault Research & Commerce Operations Portal

## Run
```bash
docker compose up --build
```
Open http://localhost:3000  (.env auto-created from .env.example on first run)

## Test
```bash
docker compose --profile test run --build test
```

## Stop
```bash
docker compose down
```

## Login

| Role            | Username | Password           |
|-----------------|----------|--------------------|
| Administrator   | admin    | ScholarAdmin2024!  |
| Content Curator | curator  | Scholar2024!       |
| Reviewer        | reviewer | Scholar2024!       |
| Finance Manager | finance  | Scholar2024!       |
| Store Manager   | store    | Scholar2024!       |
