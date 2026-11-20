# SRS - Ứng dụng Quản lý Công việc Cá nhân (FlowS)

Software Requirements Specification

## 1. Giới thiệu

FlowS là ứng dụng desktop đa nền tảng giúp người dùng quản lý kế hoạch học tập và công việc cá nhân một cách linh hoạt, tập trung, với giao diện trực quan tạo cảm giác thành tựu thông qua cơ chế gamification (vườn hoa task).

## 2. Mục tiêu và phạm vi

- Giúp người dùng theo dõi task hằng ngày, deadline, mức độ ưu tiên.
- Tạo động lực hoàn thành task thông qua hình ảnh hoá trực quan (hoa, bó hoa, heatmap).
- Tích hợp dữ liệu thời khóa biểu và deadline từ hệ thống của trường.
- Hỗ trợ quản lý nhiều dự án song song thông qua hệ thống tag.
- Không cần backend server; dữ liệu lưu trữ cục bộ (local-first).

## 3. Đối tượng người dùng

Sinh viên và cá nhân cần công cụ quản lý task/deadline kết hợp lịch học, ưu tiên trải nghiệm trực quan, thẩm mỹ hơn các to-do list truyền thống.

## 4. Kiến trúc hệ thống

```
FlowS/
├── frontend/            Vue 3 + TypeScript
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── services/
│   │   ├── database/
│   │   └── platform/ (linux.rs, windows.rs, macos.rs)
│   └── migrations/
├── packaging/ (arch, debian, fedora, appimage)
└── README.md
```

Công nghệ: Vue 3 + TypeScript + Tauri 2 + Rust + SQLite. Không backend server. Phát triển/kiểm thử chính trên Arch Linux, hỗ trợ Wayland và X11, mở rộng sang Windows/macOS và các distro Linux khác.

| Công nghệ | Vai trò | Trạng thái |
|---|---|---|
| Vue 3 | Xây dựng giao diện | Giữ nguyên |
| TypeScript | Ngôn ngữ cho Frontend | Giữ nguyên |
| Tauri 2 | Đóng gói ứng dụng Desktop | Giữ nguyên |
| Rust | Backend, xử lý nghiệp vụ | Giữ nguyên |
| SQLite | Lưu trữ dữ liệu | Giữ nguyên |
| Tailwind CSS | Styling, layout, responsive | Bổ sung |
| NES.css | Các thành phần giao diện phong cách 8-bit | Bổ sung |

## 5. Yêu cầu chức năng

### 5.1 Trang 1 - Dashboard

- Hiển thị ngày, giờ hiện tại.
- Widget media player hệ thống (thu gọn, hiển thị bài đang phát).
- Streak: số ngày liên tiếp hoàn thành ít nhất 1 task.
- Số lượng task còn lại trong ngày.
- Số task đã hoàn thành trong ngày / tổng số task của ngày.
- Activity heatmap (dạng lịch nhiệt, thể hiện mức độ hoạt động theo ngày, tương tự GitHub contribution graph).

### 5.2 Trang 2 - Danh sách Task

- Hiển thị danh sách toàn bộ task (có thể lọc/sắp xếp).
- Tạo task mới với các trường: tên task, ngày tạo(auto today), deadline, mức độ quan trọng (extreme, high, mid, low, simple).
- Sửa/xóa task.
- Đánh dấu hoàn thành task.

### 5.3 Trang 3 - Lake of Task
- Màn hình chia 2 vùng trên/dưới: chia vùng trên chiếm 3/4 màn hình, còn lại là phần dưới

- **Vùng trên**: mỗi task là con cá trong bể trời.
  - Cá chưa có màu sắc ban đầu.
  - Cá càng buồn(sẽ có màu đen dần) khi deadline càng gần.
  - Kích thước cá tương ứng mức độ quan trọng của task.
- **Tương tác**: người dùng kéo (bắt) một bé cá ra khỏi bể, animation Cá bơi xuống vùng dưới để biểu thị task đang tập trung xử lý, có thể kéo nhiều nhất 3 cá xuống.
- **Hoàn thành task**: nút "Đã xong" — khi bấm, cá được tô màu rực rỡ trở lại và trở lại bể cá ban đầu, tô thêm màu sắc cho bể.
- **Reset hàng ngày**: mỗi ngày mới, những con cá đã được tô màu sẽ không xuất hiện trong bể task, bể cá sẽ quay lại là những chú cá chỉ có màu sắc từ xám đến đen.

## 5.4 Trang 4 - Lịch sử cá đã tô màu theo tháng

- View dạng scrollbar ngang, chiếm 3/5 màn hình, căn giữa.
- Hiển thị 12 bể cá theo tháng chứa những con cá đã hoàn thành việc tô màu trong tháng đó. hiện số lượng hoàn thành task trong tháng (December - 67 tasks)

### 5.5 Trang 5 - Calendar

- Hiển thị lịch theo tuần/tháng tích hợp task và sự kiện.
- **Crawling thời khóa biểu & deadline từ hệ thống trường**:
  - Module thu thập dữ liệu (crawler/scraper) để đồng bộ thời khóa biểu học và deadline bài tập/môn học.
  - Lên lịch đồng bộ định kỳ (background job) trong github, có tùy chọn đồng bộ thủ công.
  - Ánh xạ (map) dữ liệu crawl được thành task/sự kiện trong hệ thống, tránh trùng lặp khi đồng bộ lại.
  - Cấu hình thông tin đăng nhập trường (lưu trữ an toàn, mã hóa cục bộ) để crawler truy cập hệ thống trường.
  - Hiển thị kết hợp: task cá nhân + lịch học + deadline trường trên cùng giao diện calendar.

### 5.6 Hệ thống Tag dự án (Project Tags)

- Cho phép gắn tag để phân loại task theo dự án/ngữ cảnh khác nhau (ví dụ: Học tập, Công việc, Dự án cá nhân X).
- Một task có thể gắn nhiều tag.
- Lọc/xem task theo tag ở trang danh sách, dashboard và calendar.
- Quản lý tag: tạo, đổi tên, đổi màu, xóa tag.
- Thống kê số lượng task hoàn thành theo từng tag/dự án.

### 5.7 Notification (Thông báo)

- Thông báo nhắc nhở deadline sắp đến (có thể tùy chỉnh mốc thời gian nhắc: trước 1 ngày, 1 giờ,...).
- Thông báo khi đồng bộ calendar trường thành công/thất bại.
- Thông báo streak sắp bị mất (chưa hoàn thành task nào trong ngày).
- Thông báo native hệ điều hành (qua Tauri notification API), hoạt động trên Linux/Windows/macOS.
- Cho phép bật/tắt từng loại thông báo trong phần cài đặt.

## 6. Yêu cầu phi chức năng

- **Hiệu năng**: ứng dụng nhẹ, khởi động nhanh nhờ Tauri (so với Electron).
- **Offline-first**: hoạt động đầy đủ không cần mạng, trừ tính năng crawling calendar trường.
- **Bảo mật**: thông tin đăng nhập trường được mã hóa khi lưu cục bộ.
- **Đa nền tảng**: Linux (ưu tiên, đóng gói native Arch/Debian/Fedora/AppImage), mở rộng Windows và macOS.
- **Khả năng mở rộng**: kiến trúc tách lớp commands/services/database/platform cho phép bổ sung tính năng dễ dàng. đặc biệt là bổ sung tính năng theme cho phép thay theme của app (có thể mod được toàn bộ UI chỉ cần theo rule của app)

## 7. Giả định và ràng buộc

- Hệ thống trường cần hỗ trợ truy cập dữ liệu thời khóa biểu/deadline (qua API chính thức hoặc crawling giao diện web) — cần khảo sát thêm cấu trúc hệ thống trường cụ thể để thiết kế crawler phù hợp.
- SQLite dùng làm cơ sở dữ liệu cục bộ duy nhất, không backend server tập trung.
- Phiên bản đầu ưu tiên phát triển và kiểm thử trên Arch Linux.

## Các lưu ý về thiết kế
- Style: Sans-serif, Black and white, Pattern, Monospaced, Lines, Outline, Pixelated, pixel game
- Colout: White, Black, Green (other colors is for flowers)
- Font: Geist, Geist Mono
