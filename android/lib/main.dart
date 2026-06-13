import 'package:flutter/material.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_database/firebase_database.dart';
import 'firebase_options.dart';
import 'classroom.dart';
import 'classroom_card.dart';
import 'l10n/app_localizations.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Firebase.initializeApp(options: DefaultFirebaseOptions.currentPlatform);
  runApp(const MainApp());
}

class MainApp extends StatelessWidget {
  const MainApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      theme: ThemeData.light().copyWith(cardColor: const Color(0xFFFFFFFF)),
      darkTheme: ThemeData.dark().copyWith(cardColor: const Color(0xFF2D2D2D)),
      themeMode: ThemeMode.system,
      home: const MainClassroomsScreen(),
    );
  }
}

class MainClassroomsScreen extends StatefulWidget {
  const MainClassroomsScreen({super.key});

  @override
  State<MainClassroomsScreen> createState() => _MainClassroomsScreenState();
}

class _MainClassroomsScreenState extends State<MainClassroomsScreen> {
  final DatabaseReference _camerasRef = FirebaseDatabase.instance.ref().child(
    'cameras',
  );

  @override
  Widget build(BuildContext context) {
    final t = AppLocalizations.of(context)!;
    return Scaffold(
      appBar: AppBar(
        centerTitle: true,
        title: const Text(
          'Umbral - UNESUM',
          style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20),
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              t.classroomStatus,
              style: TextStyle(fontSize: 26, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            Expanded(
              child: StreamBuilder<DatabaseEvent>(
                stream: _camerasRef.onValue,
                builder: (context, snapshot) {
                  if (snapshot.connectionState == ConnectionState.waiting) {
                    return const Center(child: CircularProgressIndicator());
                  }

                  if (snapshot.hasError) {
                    return Center(
                      child: Text(
                        'Connection error: ${snapshot.error}',
                        style: const TextStyle(color: Colors.redAccent),
                      ),
                    );
                  }

                  final dataSnapshot = snapshot.data?.snapshot;

                  if (dataSnapshot == null || dataSnapshot.value == null) {
                    return const Center(
                      child: Text(
                        'No Classrooms Added',
                        style: TextStyle(
                          color: Colors.grey,
                          fontSize: 16,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    );
                  }

                  final Map<dynamic, dynamic> camerasMap =
                      dataSnapshot.value as Map<dynamic, dynamic>;
                  final List<Classroom> classroomsList = [];

                  camerasMap.forEach((key, value) {
                    if (value is Map) {
                      classroomsList.add(
                        Classroom.fromSnapshot(key.toString(), value),
                      );
                    }
                  });

                  return GridView.builder(
                    itemCount: classroomsList.length,
                    gridDelegate:
                        const SliverGridDelegateWithMaxCrossAxisExtent(
                          maxCrossAxisExtent: 320,
                          mainAxisSpacing: 16,
                          crossAxisSpacing: 16,
                          mainAxisExtent: 220,
                        ),
                    itemBuilder: (context, index) {
                      return ClassroomCard(classroom: classroomsList[index]);
                    },
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
