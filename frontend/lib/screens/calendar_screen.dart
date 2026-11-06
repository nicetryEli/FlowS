import 'package:flutter/material.dart';

import '../data/mock_events.dart';
import '../theme.dart';
import '../views/month_view.dart';
import '../views/week_view.dart';
import '../widgets/add_event_dialog.dart';
import '../widgets/app_rail.dart';
import '../widgets/footer_strip.dart';
import '../widgets/next_up_panel.dart';
import '../widgets/top_bar.dart';
import '../views/year_view.dart';

/// Root calendar screen: icon rail · (top bar + active view) · next-up panel,
/// with the footer strip below. Mirrors the approved prototype.
class CalendarScreen extends StatefulWidget {
  const CalendarScreen({super.key});

  @override
  State<CalendarScreen> createState() => _CalendarScreenState();
}

class _CalendarScreenState extends State<CalendarScreen> {
  final MockEventsRepository _repository = MockEventsRepository();

  // Anchor defaults to the real "now" (Nov 2026 matches the mock dataset).
  CalendarViewMode _mode = CalendarViewMode.month;
  DateTime _anchor = DateTime.now();

  void _step(int delta, {int days = 0}) {
    setState(() {
      if (days != 0) {
        _anchor = _anchor.add(Duration(days: days));
      } else if (_mode == CalendarViewMode.year) {
        _anchor = DateTime(_anchor.year + delta, _anchor.month, 1);
      } else {
        _anchor = DateTime(_anchor.year, _anchor.month + delta, 1);
      }
    });
  }

  void _today() => setState(() => _anchor = DateTime.now());

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppColors.bg,
      body: Container(
        decoration: appBackground,
        child: Column(
        children: [
          Expanded(
            child: ListenableBuilder(
              listenable: _repository,
              builder: (context, _) => Row(
                children: [
                  const AppRail(),
                  Expanded(
                    child: Column(
                      children: [
                        TopBar(
                          mode: _mode,
                          onModeChanged: (m) => setState(() => _mode = m),
                          anchorDate: _anchor,
                          onPrev: () => _mode == CalendarViewMode.week ? _step(0, days: -7) : _step(-1),
                          onNext: () => _mode == CalendarViewMode.week ? _step(0, days: 7) : _step(1),
                          onToday: _today,
                        ),
                        Expanded(
                          child: AnimatedSwitcher(
                            duration: const Duration(milliseconds: 260),
                            transitionBuilder: (child, animation) => FadeTransition(
                              opacity: animation,
                              child: SlideTransition(
                                position: Tween<Offset>(begin: const Offset(0, 0.035), end: Offset.zero)
                                    .animate(CurvedAnimation(parent: animation, curve: Curves.easeOutCubic)),
                                child: child,
                              ),
                            ),
                            child: KeyedSubtree(
                              key: ValueKey('${_mode.index}-${_anchor.year}-${_anchor.month}-${_anchor.day}'),
                              child: switch (_mode) {
                                CalendarViewMode.month => MonthView(
                                    repository: _repository,
                                    visibleMonth: DateTime(_anchor.year, _anchor.month, 1),
                                    onQuickAdd: (date) => showAddEventDialog(context, _repository, initialDate: date),
                                  ),
                                CalendarViewMode.week => WeekView(repository: _repository, weekAnchor: _anchor),
                                CalendarViewMode.year => YearView(
                                    repository: _repository,
                                    year: _anchor.year,
                                    onSelectMonth: (d) => setState(() {
                                          _anchor = d;
                                          _mode = CalendarViewMode.month;
                                        }),
                                  ),
                              },
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                  NextUpPanel(
                    repository: _repository,
                    onAddPersonal: () => showAddEventDialog(context, _repository),
                  ),
                ],
              ),
            ),
          ),
          const FooterStrip(),
        ],
      ),
      ),
    );
  }
}
