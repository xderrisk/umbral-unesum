class Classroom {
  final String id;
  final String name;
  final String status;
  final bool isAvailable;
  final String? mac;
  final int? lastConnection;

  Classroom({
    required this.id,
    required this.name,
    required this.status,
    required this.isAvailable,
    this.mac,
    this.lastConnection,
  });

  factory Classroom.fromSnapshot(String id, Map<dynamic, dynamic> data) {
    final String statusValue = data['status']?.toString() ?? '0';

    bool localIsAvailable;
    String localStatusLabel;

    if (statusValue == '0') {
      localIsAvailable = true;
      localStatusLabel = 'free';
    } else if (statusValue == '1') {
      localIsAvailable = false;
      localStatusLabel = 'occupied';
    } else {
      localIsAvailable = false;
      localStatusLabel = 'offline';
    }

    int? lastConnectionTimestamp;
    if (data['last_connection'] != null) {
      if (data['last_connection'] is int) {
        lastConnectionTimestamp = data['last_connection'];
      } else if (data['last_connection'] is String) {
        lastConnectionTimestamp = int.tryParse(data['last_connection']);
      }
    }

    return Classroom(
      id: id,
      name: data['name']?.toString() ?? 'Unnamed classroom',
      status: localStatusLabel,
      isAvailable: localIsAvailable,
      mac: data['mac']?.toString(),
      lastConnection: lastConnectionTimestamp,
    );
  }
}
