# TDTU Local Calendar Dashboard

Lấy lịch học, lịch thi và deadline eLearning từ cổng TDTU, lưu vào **SQLite local** và xem trên **dashboard web (Flutter Web + FastAPI)**. Hoàn toàn chạy local, không dùng Google Calendar.

## Luồng hoạt động

```text
run_hour.py
  └─ crawler.py (TDTU portal / eLearning)
       └─ SQLite (source of truth)          ← Step 3–4: ingest + ERD
            └─ FastAPI → Flutter Web        ← Step 5: calendar UI
```

- Lịch học: class sessions từ cổng sinh viên TDTU.
- Lịch thi: exam sessions (midterm/final), kèm scrape fallback từ eLearning nếu portal trống.
- Deadline: eLearning deadlines chưa hoàn thành.

## Cài đặt local

1. Tạo môi trường Python và cài dependencies:

   ```bash
   python -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   playwright install chromium
   ```

2. Copy `.env.example` thành `.env` và điền:

   ```dotenv
   STUDENT_ID=...
   PASSWORD=...
   ```

3. Chạy pipeline:

   ```bash
   python run_hour.py
   ```

   Hiện tại crawler chạy độc lập và log số dòng thu được; bước ingest vào SQLite sẽ được thêm ở Step 3–4.

## Kiểm tra

```bash
pytest -q
python -m py_compile *.py
```

## Ghi chú vận hành

- `.env` không được commit (đã gitignore).
- Không còn phụ thuộc Google / ntfy; mọi dữ liệu nằm trong file SQLite local (đường dẫn sẽ được định nghĩa ở Step 4).
- CI (GitHub Actions) sẽ được thêm lại khi bước ingest SQLite hoàn tất.
