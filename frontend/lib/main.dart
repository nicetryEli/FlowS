import 'package:flutter/material.dart';

import 'data/events_repository.dart';
import 'data/mock_events.dart';
import 'data/api_events_repository.dart';
import 'screens/calendar_screen.dart';
import 'theme.dart';



const _useMock = bool.fromEnvironment('USE_MOCK', defaultValue: false);
const _apiBase = String.fromEnvironment('API_BASE', defaultValue: '');

void main() {
  runApp(const _BootstrapApp());
}

/// Public root used by tests and by the bootstrap. Renders the calendar
/// against the supplied repository.
class TdtuCalendarApp extends StatelessWidget {
  const TdtuCalendarApp({super.key, required this.repository});

  final CalendarEventsRepository repository;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'TDTU · Scheduled',
      debugShowCheckedModeBanner: false,
      home: CalendarScreen(repository: repository),
      theme: ThemeData(
        fontFamily: 'JetBrainsMono',
        useMaterial3: true,
      ),
    );
  }
}

/// Loads the live API repository, then hands off to [TdtuCalendarApp].
class _BootstrapApp extends StatefulWidget {
  const _BootstrapApp();

  @override
  State<_BootstrapApp> createState() => _BootstrapAppState();
}

class _BootstrapAppState extends State<_BootstrapApp> {
  CalendarEventsRepository? _repository;
  Object? _error;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      if (_useMock) {
        _repository = MockEventsRepository();
      } else {
        _repository = await ApiEventsRepository.create(baseUrl: _apiBase);
      }
    } catch (error) {
      _error = error;
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const MaterialApp(
        debugShowCheckedModeBanner: false,
        home: _LoadingScreen(),
      );
    }
    if (_error != null) {
      return MaterialApp(
        debugShowCheckedModeBanner: false,
        theme: buildAppTheme(),
        home: _ErrorScreen(error: _error!, onRetry: _load),
      );
    }
    return TdtuCalendarApp(repository: _repository!);
  }
}

class _LoadingScreen extends StatelessWidget {
  const _LoadingScreen();

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      backgroundColor: AppColors.bg,
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircularProgressIndicator(color: AppColors.blue, strokeWidth: 2.5),
            SizedBox(height: 14),
            Text('Loading your schedule…', style: TextStyle(color: AppColors.muted, fontSize: 13)),
          ],
        ),
      ),
    );
  }
}

class _ErrorScreen extends StatelessWidget {
  const _ErrorScreen({required this.error, required this.onRetry});

  final Object error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppColors.bg,
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 420),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.cloud_off_rounded, size: 42, color: AppColors.muted),
              const SizedBox(height: 16),
              const Text(
                'Cannot reach the local calendar API',
                textAlign: TextAlign.center,
                style: TextStyle(color: AppColors.white, fontSize: 17, fontWeight: FontWeight.w700),
              ),
              const SizedBox(height: 8),
              Text(
                'Make sure the backend is running:\n\n  LOCAL_DB_PATH=data/app.db .venv/bin/uvicorn server:app --port 8000\n\n($error)',
                textAlign: TextAlign.center,
                style: const TextStyle(color: AppColors.muted, fontSize: 11.5, height: 1.5),
              ),
              const SizedBox(height: 18),
              FilledButton.icon(
                onPressed: onRetry,
                icon: const Icon(Icons.refresh_rounded, size: 16),
                label: const Text('Retry'),
                style: FilledButton.styleFrom(
                  backgroundColor: AppColors.slate,
                  foregroundColor: AppColors.white,
                  padding: const EdgeInsets.symmetric(horizontal: 22, vertical: 11),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
