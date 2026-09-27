// Kịch bản dọn dẹp hệ thống chuyên sâu
// Tác giả: MemScript-AI

log "Bắt đầu quét và dọn dẹp hệ thống chuyên sâu..."

// 1. Dọn rác ổ đĩa (%TEMP%, CrashDumps, Update Cache, Thùng rác)
clean disk.temp

// 2. Dọn Shader Cache của GPU
clean gpu.shader

// 3. Xả Standby Cache của RAM
clean ram.standby

// 4. Thu hồi bộ nhớ Working Set
clean ram.workingset

log "Hệ thống đã được dọn sạch sẽ!"
