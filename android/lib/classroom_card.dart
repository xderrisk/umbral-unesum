import 'dart:async';
import 'package:flutter/material.dart';
import 'classroom.dart';
import 'l10n/app_localizations.dart';

class ClassroomCard extends StatefulWidget {
  final Classroom classroom;

  const ClassroomCard({super.key, required this.classroom});

  @override
  State<ClassroomCard> createState() => _ClassroomCardState();
}

class _ClassroomCardState extends State<ClassroomCard> {
  late Timer _timer;

  @override
  void initState() {
    super.initState();
    _timer = Timer.periodic(const Duration(seconds: 30), (timer) {
      if (mounted) setState(() {});
    });
  }

  @override
  void didUpdateWidget(ClassroomCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.classroom.lastConnection != oldWidget.classroom.lastConnection) {
      setState(() {});
    }
  }

  @override
  void dispose() {
    _timer.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = AppLocalizations.of(context)!;
    final now = DateTime.now().millisecondsSinceEpoch;
    final int? lastConn = widget.classroom.lastConnection;

    final bool isOnline = lastConn != null && (now - lastConn) < (60 * 1000);

    final Color statusColor = !isOnline
        ? Colors.grey
        : (widget.classroom.isAvailable
              ? const Color(0xFF2ec27e)
              : const Color(0xFFe01b24));

    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).cardColor,
        borderRadius: BorderRadius.circular(20),
        border: Border.all(
          color: Theme.of(context).dividerColor.withValues(alpha: 0.5),
          width: 1.5,
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.5),
            blurRadius: 6,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Stack(
        children: [
          Positioned(
            top: 14,
            right: 14,
            child: Container(
              width: 14,
              height: 14,
              decoration: BoxDecoration(
                color: statusColor,
                shape: BoxShape.circle,
              ),
            ),
          ),
          Center(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16.0),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(
                    widget.classroom.name,
                    style: const TextStyle(
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                      letterSpacing: -0.5,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    !isOnline
                        ? t.offline
                        : (widget.classroom.isAvailable ? t.free : t.occupied),
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.w700,
                      color: statusColor,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
