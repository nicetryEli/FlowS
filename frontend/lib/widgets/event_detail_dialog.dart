import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../data/events_repository.dart';
import '../models/calendar_event.dart';
import '../theme.dart';
import 'event_style.dart';

const _statusVi = {
  SessionStatus.scheduled: 'Học bình thường',
  SessionStatus.absent: 'Báo vắng',
  SessionStatus.makeup: 'Học bù',
  SessionStatus.cancelled: 'Đã hủy',
  SessionStatus.moved: 'Đã chuyển lịch',
};

Future<void> showEventDetailDialog(
  BuildContext context,
  CalendarEvent event,
  CalendarEventsRepository repository,
) {
  return showGeneralDialog<void>(
    context: context,
    barrierDismissible: true,
    barrierColor: Colors.black.withValues(alpha: 0.55),
    transitionDuration: const Duration(milliseconds: 230),
    pageBuilder: (context, animation, secondaryAnimation) =>
        _EventDetailDialog(event: event, repository: repository),
    transitionBuilder: (context, animation, secondaryAnimation, child) {
      final curved = CurvedAnimation(parent: animation, curve: Curves.easeOutCubic);
      return FadeTransition(
        opacity: curved,
        child: ScaleTransition(
          scale: Tween<double>(begin: 0.94, end: 1.0).animate(curved),
          child: child,
        ),
      );
    },
  );
}

class _EventDetailDialog extends StatelessWidget {
  const _EventDetailDialog({required this.event, required this.repository});

  final CalendarEvent event;
  final CalendarEventsRepository repository;

  @override
  Widget build(BuildContext context) {
    final accent = accentFor(event);
    final rows = _detailRows(event);

    return Dialog(
      backgroundColor: const Color(0xFF232E30),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(AppRadii.control + 4)),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 460, minWidth: 360),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(22, 18, 12, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                    decoration: BoxDecoration(
                      color: accent.withValues(alpha: 0.13),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      kindLabel(event),
                      style: TextStyle(
                        color: accent,
                        fontSize: 9.5,
                        fontWeight: FontWeight.w800,
                        letterSpacing: 1.4,
                      ),
                    ),
                  ),
                  const Spacer(),
                  IconButton(
                    icon: const Icon(Icons.close_rounded, color: AppColors.muted, size: 20),
                    onPressed: () => Navigator.of(context).pop(),
                    splashRadius: 18,
                  ),
                ],
              ),
              const SizedBox(height: 4),
              Text(
                event.title,
                style: const TextStyle(color: AppColors.text, fontSize: 19, fontWeight: FontWeight.w700, height: 1.25),
              ),
              const SizedBox(height: 14),
              ...rows,
              const SizedBox(height: 14),
              const Divider(height: 1, color: Color(0x14FFFFFF)),
              const SizedBox(height: 10),
              Row(
                children: [
                  _dialogButton(context, Icons.edit_outlined, 'Edit'),
                  const SizedBox(width: 8),
                  _dialogButton(
                    context,
                    Icons.delete_outline_rounded,
                    'Delete',
                    onTap: () async {
                      await _confirmDelete(context);
                    },
                  ),
                  const Spacer(),
                  if (event.kind == CalendarEventKind.deadline && event.activityUrl != null)
                    FilledButton.icon(
                      onPressed: () => launchUrl(Uri.parse(event.activityUrl!), mode: LaunchMode.externalApplication),
                      icon: const Icon(Icons.open_in_new_rounded, size: 15),
                      label: const Text('Open in eLearning'),
                      style: FilledButton.styleFrom(
                        backgroundColor: AppColors.slate,
                        foregroundColor: AppColors.white,
                        textStyle: const TextStyle(fontSize: 12, fontWeight: FontWeight.w800),
                        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                      ),
                    ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  List<Widget> _detailRows(CalendarEvent event) {
    String dateLine = '${_longDate(event.start)} · ${weekdayLong(event.start)}';
    final rows = <Widget>[_row('DATE', dateLine)];

    switch (event.kind) {
      case CalendarEventKind.classSession:
        rows.add(_row(
          'TIME',
          '${event.start.hour.toString().padLeft(2, '0')}:${event.start.minute.toString().padLeft(2, '0')} – '
          '${event.end.hour.toString().padLeft(2, '0')}:${event.end.minute.toString().padLeft(2, '0')}'
          '${event.startPeriod != null ? '  ·  Tiết ${event.startPeriod}–${event.endPeriod}' : ''}',
        ));
        rows.add(_row('ROOM', event.location ?? '—'));
        rows.add(_row('STATUS', _statusVi[event.status] ?? 'Học bình thường'));
      case CalendarEventKind.exam:
        rows.add(_row('TIME', event.allDay ? 'Cả ngày' : timeRange(event)));
        if (event.location != null) rows.add(_row('ROOM', event.location!));
        rows.add(_row('TYPE', event.examType ?? '—'));
      case CalendarEventKind.deadline:
        rows.add(_row('DUE', '${_longDate(event.start)} · ${event.start.hour.toString().padLeft(2, '0')}:${event.start.minute.toString().padLeft(2, '0')} (GMT+7)'));
        rows.add(_row('COURSE', event.courseName ?? '—'));
        rows.add(_row('STATUS', 'Chưa hoàn thành'));
      case CalendarEventKind.personal:
        if (event.allDay) {
          rows.add(_row('WHEN', '${_longDate(event.start)} → ${_longDate(event.end)}'));
          rows.add(_row('TYPE', 'All-day · multi-day'));
        } else {
          rows.add(_row('TIME', timeRange(event)));
        }
    }
    if (event.notes != null && event.notes!.isNotEmpty) {
      rows.add(_row('NOTES', event.notes!));
    }
    return rows;
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 78,
            child: Text(
              label,
              style: const TextStyle(
                color: AppColors.muted2,
                fontSize: 9,
                fontWeight: FontWeight.w800,
                letterSpacing: 1.3,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(color: AppColors.text, fontSize: 12, height: 1.35),
            ),
          ),
        ],
      ),
    );
  }

  Widget _dialogButton(BuildContext context, IconData icon, String label, {VoidCallback? onTap}) {
    return TextButton.icon(
      onPressed: onTap ?? () => ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$label — wiring comes with the FastAPI backend (PATCH /api/events)'), duration: const Duration(seconds: 2)),
          ),
      icon: Icon(icon, size: 15, color: AppColors.muted),
      label: Text(label, style: const TextStyle(color: AppColors.text, fontSize: 12)),
      style: TextButton.styleFrom(
        backgroundColor: const Color(0xFF2A3537),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        minimumSize: const Size(0, 34),
      ),
    );
  }

  Future<void> _confirmDelete(BuildContext context) async {
    final ok = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xFF232E30),
        title: const Text('Delete this event?', style: TextStyle(color: AppColors.text, fontSize: 16)),
        content: Text(event.title, style: const TextStyle(color: AppColors.muted, fontSize: 13)),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(AppRadii.control + 2)),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete', style: TextStyle(color: AppColors.red)),
          ),
        ],
      ),
    );
    if (ok == true && context.mounted) {
      try {
        await repository.remove(event.id);
        if (context.mounted) Navigator.of(context).pop();
      } catch (error) {
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Could not delete: $error')));
        }
      }
    }
  }

  static String _longDate(DateTime d) =>
      '${d.day} ${_months[d.month - 1]} ${d.year}';
}

const _months = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
];
