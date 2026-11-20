## Mô tả dự án

**FlowS** là ứng dụng desktop quản lý công việc/học tập cá nhân cho sinh viên, chạy local-first (không backend server, dữ liệu lưu SQLite). Điểm khác biệt so với to-do list thường là gamification: mỗi task là một con cá trong bể. Cá càng gần deadline càng xám/đen, task xong thì cá được tô màu rực rỡ và bể đầy màu dần.

**Stack:** Vue 3 + TypeScript (UI), Tauri 2 + Rust (logic, DB, native), SQLite, Tailwind + NES.css (phong cách pixel, đen trắng + xanh biển, font Geist).

**Các phần chính:**

1. Dashboard: giờ, media player, streak, tiến độ ngày, heatmap
2. Danh sách task: CRUD, lọc/sắp xếp, 5 mức quan trọng
3. Lake of Task: bể cá, kéo tối đa 3 cá xuống vùng "đang làm", nút "Đã xong", reset màu mỗi ngày
4. Lịch sử theo tháng: 12 bể cá cuộn ngang
5. Calendar: tuần/tháng + crawler thời khóa biểu/deadline từ trường
6. Tag dự án + thống kê
7. Notification native

## Kế hoạch code

**Giai đoạn 0: Nền tảng**

- Khởi tạo Tauri 2 + Vue 3 + TS, cài Tailwind, NES.css, font Geist.
- Dựng cấu trúc `commands/services/database` trong Rust, `vue-router` cho 5 trang, Pinia cho state.
- Thiết lập SQLite (sqlx hoặc rusqlite) + hệ thống migrations.
- Đặt sẵn **design tokens bằng CSS variables** (màu, font). Việc này giúp tính năng theme về sau gần như không tốn thêm công.

**Giai đoạn 1: Core task**

- Schema: `tasks`, `tags`, `task_tags`, `completions` (log hoàn thành theo ngày, dùng cho streak/heatmap/lịch sử).
- Rust commands: create/update/delete/complete/list task; frontend gọi qua `invoke`.
- Trang 2 (Task list) hoàn chỉnh, gắn tag cơ bản.
- Tách rõ: dữ liệu bền vững lưu DB, trạng thái "reset hằng ngày" và "cá đang focus" tính từ `completed_at` chứ không ghi đè dữ liệu.

**Giai đoạn 2: Dashboard**

- Query streak, task còn lại/đã xong hôm nay.
- Heatmap kiểu GitHub từ bảng `completions` (tự vẽ bằng grid CSS).
- Media player: dùng MPRIS (D-Bus) trên Linux, để trait `platform/` cho Windows/macOS làm sau.

**Giai đoạn 3: Lake of Task**

- Render cá bằng canvas hoặc DOM + sprite pixel; kích thước theo độ quan trọng, màu xám→đen theo công thức `(now - created) / (deadline - created)`.
- Kéo thả (pointer events), animation bơi xuống vùng dưới, giới hạn 3 cá.
- Nút "Đã xong": tô màu, đưa cá về bể, ghi `completions`.
- Logic reset ngày mới (kiểm tra khi mở app và khi qua nửa đêm).
- Làm bản sprite đơn giản trước, thay art đẹp sau.

**Giai đoạn 4: Lịch sử tháng**

- 12 bể cá, scroll ngang, query đếm theo tháng (ví dụ: "December - 67 tasks"). Tái sử dụng component bể cá của Giai đoạn 3.

**Giai đoạn 5: Calendar**

- Hiển thị tuần/tháng gộp task cá nhân + lịch học + deadline trường.
- Thêm cột `source` và `external_id` vào task/event để chống trùng khi sync lại.

**Giai đoạn 6: Crawler trường**

- Viết trong Rust (`reqwest` + `scraper`), đồng bộ thủ công trước, định kỳ sau.
- Lưu credential bằng keyring hệ điều hành (crate `keyring`) thay vì tự mã hóa file. Cách này an toàn hơn.
- Lưu ý: SRS ghi "background job trong github" nhưng app không có server. Nên hiểu là job nền trong app (tokio scheduler) khi app đang chạy, hay GitHub Actions? Cần quyết định, vì GitHub Actions sẽ phá nguyên tắc local-first và lộ credential.

**Giai đoạn 7: Notification + Cài đặt (3-4 ngày)**

- Tauri notification plugin, scheduler nhắc deadline, cảnh báo mất streak, bật/tắt từng loại.

**Giai đoạn 8: Theme, đóng gói, hoàn thiện (1 tuần)**

- Theme: nạp file JSON/CSS ghi đè design tokens.
- Đóng gói Arch (PKGBUILD), Debian, Fedora, AppImage; test Wayland/X11.

## Gợi ý thứ tự ưu tiên

Làm **MVP = Giai đoạn 0 → 3** trước (task + dashboard + bể cá), vì đây là phần tạo nên bản sắc của app. Crawler để sau cùng vì phụ thuộc hệ thống trường bên ngoài.

## Vài điểm lưu ý

- Cá "kéo xuống" có lưu qua lần mở app 
- Task quá hạn mà chưa xong thì cá hiển thị đen hẳn
- Task đồng bộ từ trường có xuất hiện thành cá trong bể 
- Task không có deadline thì màu cá sẽ là màu trắng
