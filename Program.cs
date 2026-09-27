using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Net;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

namespace MemScriptAI
{
    class Program
    {
        // ================= CONFIGURATION =================
        const string API_KEY = "sk-xt-6326e90bccc399cd6cf674b37ac9f3eef99ff46740e067bf";
        const string BASE_URL = "https://api.xkiro.com/v1/chat/completions";
        const string MODEL = "qwen/qwen3.5-omni-flash:free";
        static int AutoCleanThreshold = 85;
        static bool AutoCleanEnabled = true;

        // ================= NATIVE APIS =================
        [DllImport("ntdll.dll", SetLastError = true)]
        static extern int NtSetSystemInformation(int SystemInformationClass, IntPtr SystemInformation, int SystemInformationLength);

        [DllImport("ntdll.dll", SetLastError = true)]
        static extern int NtSetTimerResolution(uint DesiredResolution, bool SetResolution, out uint CurrentResolution);

        [DllImport("kernel32.dll")]
        static extern bool GlobalMemoryStatusEx(ref MEMORYSTATUSEX lpBuffer);

        [DllImport("Shell32.dll", CharSet = CharSet.Unicode)]
        static extern uint SHEmptyRecycleBin(IntPtr hwnd, string pszRootPath, uint dwFlags);

        [StructLayout(LayoutKind.Sequential)]
        struct MEMORYSTATUSEX
        {
            public uint dwLength;
            public uint dwMemoryLoad;
            public ulong ullTotalPhys;
            public ulong ullAvailPhys;
            public ulong ullTotalPageFile;
            public ulong ullAvailPageFile;
            public ulong ullTotalVirtual;
            public ulong ullAvailVirtual;
            public ulong ullAvailExtendedVirtual;
        }

        const int SystemMemoryListInformation = 80;
        const int MemoryEmptyWorkingSets = 2;
        const int MemoryFlushModifiedList = 3;
        const int MemoryPurgeStandbyList = 4;
        const int MemoryPurgeLowPriorityStandbyList = 5;

        static void Main(string[] args)
        {
            Console.OutputEncoding = Encoding.UTF8;
            Console.Title = "MemScript-AI v1.0 [Hardware & Memory Optimizer]";

            // Kích hoạt TLS 1.2 cho Windows WebClient
            ServicePointManager.SecurityProtocol = (SecurityProtocolType)3072 | SecurityProtocolType.Tls;

            // Kích hoạt Timer Resolution 0.5ms
            uint curRes;
            NtSetTimerResolution(5000, true, out curRes);

            PrintBanner();

            // Khởi động luồng dọn RAM ngầm
            Thread autoThread = new Thread(AutoCleanLoop) { IsBackground = true };
            autoThread.Start();

            // Vòng lặp tương tác CLI
            while (true)
            {
                Console.ForegroundColor = ConsoleColor.Cyan;
                Console.Write("\nMemScript-AI> ");
                Console.ResetColor();

                string input = Console.ReadLine();
                if (string.IsNullOrEmpty(input)) continue;
                input = input.Trim();

                string lower = input.ToLower();
                if (lower == "exit" || lower == "quit" || lower == "q")
                {
                    NtSetTimerResolution(156250, false, out curRes);
                    break;
                }
                else if (lower == "1" || lower == "clean ram" || lower == "ram")
                {
                    CleanAllRam();
                }
                else if (lower == "2" || lower == "standby")
                {
                    PurgeStandby();
                }
                else if (lower == "3" || lower == "temp" || lower == "clean disk")
                {
                    CleanDiskTemp();
                }
                else if (lower == "4" || lower == "shader" || lower == "clean gpu")
                {
                    CleanGpuShader();
                }
                else if (lower == "5" || lower == "boost" || lower == "timer")
                {
                    NtSetTimerResolution(5000, true, out curRes);
                    Console.ForegroundColor = ConsoleColor.Green;
                    Console.WriteLine("✔ [CPU] Đã mở khóa Timer Resolution 0.5ms (Giảm Input Lag tối đa)!");
                    Console.ResetColor();
                }
                else if (lower == "status")
                {
                    PrintStatus();
                }
                else if (lower == "auto")
                {
                    AutoCleanEnabled = !AutoCleanEnabled;
                    Console.WriteLine(AutoCleanEnabled ? "✔ Đã BẬT dọn RAM tự động khi vượt 85%." : "✖ Đã TẮT dọn RAM tự động.");
                }
                else
                {
                    // Lệnh ngôn ngữ tự nhiên gửi tới AI
                    AskAI(input);
                }
            }
        }

        static void CleanAllRam()
        {
            ulong beforeUsed = GetUsedRamMB();
            PurgeCommand(MemoryEmptyWorkingSets);
            PurgeCommand(MemoryPurgeStandbyList);
            PurgeCommand(MemoryPurgeLowPriorityStandbyList);
            PurgeCommand(MemoryFlushModifiedList);
            ulong afterUsed = GetUsedRamMB();
            ulong freed = (beforeUsed > afterUsed) ? (beforeUsed - afterUsed) : 0;

            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("✔ [RAM] Đã xả toàn bộ RAM! Sử dụng: {0} MB -> {1} MB (Thu hồi: {2} MB)", beforeUsed, afterUsed, freed);
            Console.ResetColor();
        }

        static void PurgeStandby()
        {
            if (PurgeCommand(MemoryPurgeStandbyList))
            {
                Console.ForegroundColor = ConsoleColor.Green;
                Console.WriteLine("✔ [RAM] Đã xả sạch Standby Cache (Chống giật lag game)!");
                Console.ResetColor();
            }
        }

        static bool PurgeCommand(int cmd)
        {
            GCHandle handle = GCHandle.Alloc(cmd, GCHandleType.Pinned);
            try
            {
                int status = NtSetSystemInformation(SystemMemoryListInformation, handle.AddrOfPinnedObject(), sizeof(int));
                return status >= 0;
            }
            finally
            {
                handle.Free();
            }
        }

        static void CleanDiskTemp()
        {
            Console.WriteLine("Đang quét và dọn rác hệ thống...");
            long totalBytes = 0;
            int totalFiles = 0;

            string temp1 = Path.GetTempPath();
            string temp2 = @"C:\Windows\Temp";
            string crashDumps = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "CrashDumps");
            string updateCache = @"C:\Windows\SoftwareDistribution\Download";

            CleanDir(temp1, ref totalFiles, ref totalBytes);
            CleanDir(temp2, ref totalFiles, ref totalBytes);
            CleanDir(crashDumps, ref totalFiles, ref totalBytes);
            CleanDir(updateCache, ref totalFiles, ref totalBytes);

            SHEmptyRecycleBin(IntPtr.Zero, null, 7); // 7 = no confirmation, no ui, no sound

            long mb = totalBytes / (1024 * 1024);
            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("✔ [DISK] Đã dọn {0} file rác/temp, giải phóng: {1} MB!", totalFiles, mb);
            Console.ResetColor();
        }

        static void CleanGpuShader()
        {
            Console.WriteLine("Đang quét và dọn Shader Cache của GPU...");
            long totalBytes = 0;
            int totalFiles = 0;
            string local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);

            CleanDir(Path.Combine(local, "D3DSCache"), ref totalFiles, ref totalBytes);
            CleanDir(Path.Combine(local, @"NVIDIA\DXCache"), ref totalFiles, ref totalBytes);
            CleanDir(Path.Combine(local, @"NVIDIA\GLCache"), ref totalFiles, ref totalBytes);
            CleanDir(Path.Combine(local, @"AMD\DxCache"), ref totalFiles, ref totalBytes);

            long mb = totalBytes / (1024 * 1024);
            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("✔ [GPU] Đã dọn {0} file Shader Cache, thu hồi: {1} MB!", totalFiles, mb);
            Console.ResetColor();
        }

        static void CleanDir(string dir, ref int count, ref long bytes)
        {
            if (!Directory.Exists(dir)) return;
            try
            {
                foreach (string file in Directory.GetFiles(dir))
                {
                    try
                    {
                        FileInfo fi = new FileInfo(file);
                        long len = fi.Length;
                        fi.Delete();
                        count++;
                        bytes += len;
                    }
                    catch { }
                }
                foreach (string sub in Directory.GetDirectories(dir))
                {
                    CleanDir(sub, ref count, ref bytes);
                    try { Directory.Delete(sub); } catch { }
                }
            }
            catch { }
        }

        static ulong GetUsedRamMB()
        {
            MEMORYSTATUSEX mem = new MEMORYSTATUSEX();
            mem.dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX));
            if (GlobalMemoryStatusEx(ref mem))
            {
                ulong total = mem.ullTotalPhys / (1024 * 1024);
                ulong avail = mem.ullAvailPhys / (1024 * 1024);
                return total - avail;
            }
            return 0;
        }

        static void PrintStatus()
        {
            MEMORYSTATUSEX mem = new MEMORYSTATUSEX();
            mem.dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX));
            if (GlobalMemoryStatusEx(ref mem))
            {
                ulong total = mem.ullTotalPhys / (1024 * 1024);
                ulong avail = mem.ullAvailPhys / (1024 * 1024);
                ulong used = total - avail;
                uint load = mem.dwMemoryLoad;

                Console.WriteLine("\n📊 [Trạng thái Phần cứng Thời Gian Thực]:");
                Console.WriteLine("   • RAM Sử Dụng   : {0} MB / {1} MB ({2}%)", used, total, load);
                Console.WriteLine("   • RAM Còn Trống : {0} MB", avail);
            }
        }

        static void AutoCleanLoop()
        {
            while (true)
            {
                Thread.Sleep(15000);
                if (AutoCleanEnabled)
                {
                    MEMORYSTATUSEX mem = new MEMORYSTATUSEX();
                    mem.dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX));
                    if (GlobalMemoryStatusEx(ref mem))
                    {
                        if (mem.dwMemoryLoad >= AutoCleanThreshold)
                        {
                            Console.ForegroundColor = ConsoleColor.Yellow;
                            Console.WriteLine("\n[Auto-Clean] RAM chạm ngưỡng {0}% (Hiện tại: {1}%), đang tự động dọn...", AutoCleanThreshold, mem.dwMemoryLoad);
                            Console.ResetColor();
                            CleanAllRam();
                            Console.Write("\nMemScript-AI> ");
                        }
                    }
                }
            }
        }

        static void AskAI(string prompt)
        {
            Console.ForegroundColor = ConsoleColor.Cyan;
            Console.WriteLine("🤖 [AI] Đang gửi yêu cầu tới model {0}...", MODEL);
            Console.ResetColor();

            try
            {
                ulong used = GetUsedRamMB();
                string sysPrompt = string.Format("Bạn là AI quản lý phần cứng Windows của MemScript-AI. RAM đang dùng: {0}MB. Hãy trả lời ngắn gọn tiếng Việt và nếu người dùng muốn dọn dẹp hoặc tối ưu, hãy hướng dẫn các lệnh: 1 (Clean RAM), 2 (Clean Standby), 3 (Clean Disk), 4 (Clean GPU Shader), 5 (Timer Boost 0.5ms).", used);

                string escapedPrompt = prompt.Replace("\"", "\\\"").Replace("\n", " ");
                string escapedSys = sysPrompt.Replace("\"", "\\\"").Replace("\n", " ");

                string jsonBody = "{\"model\":\"" + MODEL + "\",\"messages\":[{\"role\":\"system\",\"content\":\"" + escapedSys + "\"},{\"role\":\"user\",\"content\":\"" + escapedPrompt + "\"}],\"max_tokens\":512}";

                using (WebClient client = new WebClient())
                {
                    client.Encoding = Encoding.UTF8;
                    client.Headers[HttpRequestHeader.ContentType] = "application/json";
                    client.Headers[HttpRequestHeader.Authorization] = "Bearer " + API_KEY;

                    string response = client.UploadString(BASE_URL, jsonBody);

                    // Phân tích nội dung content từ JSON phản hồi
                    int contentIdx = response.IndexOf("\"content\":");
                    if (contentIdx > 0)
                    {
                        int start = response.IndexOf("\"", contentIdx + 10) + 1;
                        int end = response.IndexOf("\"", start);
                        string reply = response.Substring(start, end - start);
                        reply = reply.Replace("\\n", "\n").Replace("\\\"", "\"");

                        Console.ForegroundColor = ConsoleColor.Green;
                        Console.WriteLine("\n🤖 [AI Trả Lời]:\n{0}\n", reply);
                        Console.ResetColor();

                        // Tự động phân tích ý định dọn dẹp
                        string check = prompt.ToLower();
                        if (check.Contains("tối ưu") || check.Contains("dọn") || check.Contains("game") || check.Contains("lag") || check.Contains("ram"))
                        {
                            Console.ForegroundColor = ConsoleColor.Magenta;
                            Console.WriteLine("⚡ [AI Auto-Action] Đang tự động kích hoạt tối ưu toàn diện theo yêu cầu...");
                            Console.ResetColor();
                            CleanAllRam();
                            PurgeStandby();
                            CleanDiskTemp();
                            CleanGpuShader();
                            uint cur;
                            NtSetTimerResolution(5000, true, out cur);
                            Console.ForegroundColor = ConsoleColor.Green;
                            Console.WriteLine("✔ Đã hoàn tất toàn bộ chu trình tối ưu hóa phần cứng!");
                            Console.ResetColor();
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                Console.ForegroundColor = ConsoleColor.Red;
                Console.WriteLine("✖ [AI Lỗi]: " + ex.Message);
                Console.ResetColor();
            }
        }

        static void PrintBanner()
        {
            Console.ForegroundColor = ConsoleColor.Blue;
            Console.WriteLine("=================================================================");
            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("       MemScript-AI v1.0 - Windows Hardware & Memory Optimizer   ");
            Console.ForegroundColor = ConsoleColor.Cyan;
            Console.WriteLine("   Native Low-Level Engine + xKiro AI (Qwen3.5-Omni-Flash)       ");
            Console.ForegroundColor = ConsoleColor.Blue;
            Console.WriteLine("=================================================================");
            Console.ResetColor();
            PrintStatus();
            Console.ForegroundColor = ConsoleColor.Yellow;
            Console.WriteLine("\nDanh mục lệnh:");
            Console.ResetColor();
            Console.WriteLine("  [1] hoặc clean ram   : Dọn dẹp toàn diện RAM (WorkingSets + Standby)");
            Console.WriteLine("  [2] hoặc standby     : Xả sạch Standby Cache (Chống drop FPS game)");
            Console.WriteLine("  [3] hoặc temp        : Dọn rác hệ thống (%TEMP%, CrashDumps)");
            Console.WriteLine("  [4] hoặc shader      : Dọn GPU Shader Cache (NVIDIA / AMD / DirectX)");
            Console.WriteLine("  [5] hoặc boost       : Mở khóa Timer Resolution 0.5ms (Giảm Input Lag)");
            Console.WriteLine("  auto                 : Bật/Tắt chế độ tự động dọn RAM nền");
            Console.WriteLine("  exit                 : Thoát chương trình");
            Console.WriteLine("-----------------------------------------------------------------");
            Console.ForegroundColor = ConsoleColor.Magenta;
            Console.WriteLine("💡 Hoặc gõ lệnh tự nhiên bất kỳ, ví dụ: 'Tối ưu máy cho tao chơi game'");
            Console.ResetColor();
        }
    }
}
