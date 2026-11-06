import 'package:flutter/material.dart';

import '../data/events_repository.dart';
import '../models/calendar_event.dart';
import '../theme.dart';

const _colorOptions = <String, Color>{
  'green': AppColors.green,
  'blue': AppColors.blue,
  'yellow': AppColors.gold,
  'orange': AppColors.orange,
  'pink': AppColors.magenta,
  'purple': AppColors.purple,
  'grey': AppColors.absentAccent,
};

Future<void> showAddEventDialog(
  BuildContext context,
  CalendarEventsRepository repository, {
  DateTime? initialDate,
}) {
  return showGeneralDialog<void>(
    context: context,
    barrierDismissible: true,
    barrierColor: Colors.black.withValues(alpha: 0.55),
    transitionDuration: const Duration(milliseconds: 230),
    pageBuilder: (context, animation, secondaryAnimation) =>
        _AddEventDialog(repository: repository, initialDate: initialDate),
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

class _AddEventDialog extends StatefulWidget {
  const _AddEventDialog({required this.repository, this.initialDate});

  final CalendarEventsRepository repository;
  final DateTime? initialDate;

  @override
  State<_AddEventDialog> createState() => _AddEventDialogState();
}

class _AddEventDialogState extends State<_AddEventDialog> {
  final _titleController = TextEditingController();
  late DateTime _date = widget.initialDate ?? DateTime.now();
  bool _allDay = false;
  TimeOfDay _startTime = const TimeOfDay(hour: 9, minute: 0);
  TimeOfDay _endTime = const TimeOfDay(hour: 10, minute: 0);
  String _color = 'green';

  @override
  void dispose() {
    _titleController.dispose();
    super.dispose();
  }

  Future<void> _pickDate() async {
    final picked = await showDatePicker(
      context: context,
      initialDate: _date,
      firstDate: DateTime(2020),
      lastDate: DateTime(2035),
      builder: (context, child) => Theme(
        data: ThemeData(
          useMaterial3: true,
          colorScheme: const ColorScheme.dark(
            primary: AppColors.white,
            surface: Color(0xFF232E30),
          ),
        ),
        child: child!,
      ),
    );
    if (picked != null) setState(() => _date = picked);
  }

  Future<void> _pickTime(bool isStart) async {
    final picked = await showTimePicker(
      context: context,
      initialTime: isStart ? _startTime : _endTime,
      builder: (context, child) => Theme(
        data: ThemeData(
          useMaterial3: true,
          colorScheme: const ColorScheme.dark(
            primary: AppColors.white,
            surface: Color(0xFF232E30),
          ),
        ),
        child: child!,
      ),
    );
    if (picked != null) {
      setState(() {
        if (isStart) {
          _startTime = picked;
        } else {
          _endTime = picked;
        }
      });
    }
  }

  Future<void> _save() async {
    final title = _titleController.text.trim();
    if (title.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Give the event a title first')));
      return;
    }
    final start = _allDay
        ? DateTime(_date.year, _date.month, _date.day)
        : DateTime(_date.year, _date.month, _date.day, _startTime.hour, _startTime.minute);
    final end = _allDay
        ? DateTime(_date.year, _date.month, _date.day, 23, 59)
        : DateTime(_date.year, _date.month, _date.day, _endTime.hour, _endTime.minute);
    final event = CalendarEvent(
      id: 'personal-${DateTime.now().microsecondsSinceEpoch}',
      kind: CalendarEventKind.personal,
      title: title,
      start: start,
      end: end.isBefore(start) ? start.add(const Duration(hours: 1)) : end,
      allDay: _allDay,
      color: _color,
    );
    try {
      await widget.repository.add(event);
      if (mounted) Navigator.of(context).pop();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Could not save the event: $error')));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: const Color(0xFF232E30),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(AppRadii.control + 4)),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 440, minWidth: 360),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(22, 20, 22, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'ADD PERSONAL EVENT',
                style: TextStyle(color: AppColors.muted2, fontSize: 10, fontWeight: FontWeight.w800, letterSpacing: 1.6),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _titleController,
                autofocus: true,
                style: const TextStyle(color: AppColors.text, fontSize: 15),
                decoration: InputDecoration(
                  hintText: 'Event title…',
                  hintStyle: const TextStyle(color: AppColors.muted2),
                  border: InputBorder.none,
                  isDense: true,
                ),
                onSubmitted: (_) => _save(),
              ),
              const Divider(height: 18, color: Color(0x14FFFFFF)),
              // Date + all-day toggle
              Row(
                children: [
                  _rowButton(Icons.calendar_today_rounded,
                      '${_date.day}/${_date.month}/${_date.year}', _pickDate),
                  const SizedBox(width: 10),
                  const Text('All day', style: TextStyle(color: AppColors.muted, fontSize: 12)),
                  Switch(
                    value: _allDay,
                    onChanged: (v) => setState(() => _allDay = v),
                    activeThumbColor: AppColors.white,
                    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  ),
                ],
              ),
              if (!_allDay) ...[
                const SizedBox(height: 8),
                Row(
                  children: [
                    _rowButton(Icons.schedule_rounded, _startTime.format(context), () => _pickTime(true)),
                    const SizedBox(width: 10),
                    const Text('→', style: TextStyle(color: AppColors.muted2)),
                    const SizedBox(width: 10),
                    _rowButton(Icons.schedule_rounded, _endTime.format(context), () => _pickTime(false)),
                  ],
                ),
              ],
              const SizedBox(height: 14),
              // Color palette
              const Text('COLOR', style: TextStyle(color: AppColors.muted2, fontSize: 9, fontWeight: FontWeight.w800, letterSpacing: 1.4)),
              const SizedBox(height: 8),
              Wrap(
                spacing: 10,
                children: [
                  for (final entry in _colorOptions.entries)
                    GestureDetector(
                      onTap: () => setState(() => _color = entry.key),
                      child: Container(
                        width: 26,
                        height: 26,
                        decoration: BoxDecoration(
                          color: entry.value,
                          borderRadius: BorderRadius.circular(5),
                          boxShadow: _color == entry.key
                              ? [BoxShadow(color: entry.value.withValues(alpha: 0.5), blurRadius: 8)]
                              : null,
                        ),
                        child: _color == entry.key
                            ? const Icon(Icons.check_rounded, size: 15, color: Color(0xFF062A1C))
                            : null,
                      ),
                    ),
                ],
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(),
                    child: const Text('Cancel', style: TextStyle(color: AppColors.muted, fontSize: 12)),
                  ),
                  const Spacer(),
                  FilledButton(
                    onPressed: _save,
                    style: FilledButton.styleFrom(
                      backgroundColor: AppColors.slate,
                      foregroundColor: AppColors.white,
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(AppRadii.control)),
                      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 11),
                    ),
                    child: const Text('Save event', style: TextStyle(fontWeight: FontWeight.w800)),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _rowButton(IconData icon, String label, VoidCallback onTap) {
    return Material(
      color: const Color(0xFF2A3537),
      borderRadius: BorderRadius.circular(AppRadii.control),
      child: InkWell(
        borderRadius: BorderRadius.circular(AppRadii.control),
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 15, color: AppColors.muted),
              const SizedBox(width: 8),
              Text(label, style: const TextStyle(color: AppColors.text, fontSize: 12, fontWeight: FontWeight.w600)),
            ],
          ),
        ),
      ),
    );
  }
}
