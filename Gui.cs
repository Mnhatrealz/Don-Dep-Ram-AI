using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.IO;
using System.Net;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Windows.Forms;

namespace MemScriptAI
{
    public class MainForm : Form
    {
        // ================= CONFIGURATION =================
        const string API_KEY = "sk-xt-6326e90bccc399cd6cf674b37ac9f3eef99ff46740e067bf";
        const string BASE_URL = "https://api.xkiro.com/v1/chat/completions";
        const string MODEL = "qwen/qwen3.5-omni-flash:free";
        int autoCleanThreshold = 85;
        bool autoCleanEnabled = true;

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

        // ================= CONTROLS =================
        private System.Windows.Forms.Timer refreshTimer;
        private Label lblRamPercent;
        private Label lblRamDetails;
        private Label lblFreeRam;
        private Label lblTimerRes;
        private Panel pnlRamProgress;
        private TextBox txtAiInput;
        private RichTextBox rtbLog;
        private Button btnAiSend;
        private CheckBox chkAutoClean;
        private NotifyIcon trayIcon;

        private int currentRamLoad = 0;
        private ulong currentUsedMB = 0;
        private ulong currentTotalMB = 0;
        private ulong currentFreeMB = 0;

        [STAThread]
        public static void Main()
        {
            try
            {
                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                Application.Run(new MainForm());
            }
            catch (Exception ex)
            {
                MessageBox.Show("Lỗi khởi động: " + ex.Message, "MemScript-AI Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                File.WriteAllText("gui_error.log", ex.ToString());
            }
        }

        public MainForm()
        {
            // Thiết lập TLS 1.2
            ServicePointManager.SecurityProtocol = (SecurityProtocolType)3072 | SecurityProtocolType.Tls;

            // Kích hoạt Timer 0.5ms mặc định
            uint curRes;
            NtSetTimerResolution(5000, true, out curRes);

            InitializeComponent();
            UpdateMemoryStats();
        }

        private void InitializeComponent()
        {
            this.Text = "MemScript-AI ⚡ Hardware & Memory Optimizer";
            this.Size = new Size(880, 680);
            this.StartPosition = FormStartPosition.CenterScreen;
            this.BackColor = Color.FromArgb(16, 17, 24);
            this.ForeColor = Color.White;
            this.Font = new Font("Segoe UI", 9.5f, FontStyle.Regular);
            this.FormBorderStyle = FormBorderStyle.FixedSingle;
            this.MaximizeBox = false;

            // Header Panel
            Panel pnlHeader = new Panel
            {
                Dock = DockStyle.Top,
                Height = 65,
                BackColor = Color.FromArgb(22, 24, 34),
                Padding = new Padding(20, 10, 20, 10)
            };

            Label lblTitle = new Label
            {
                Text = "MemScript-AI ⚡",
                Font = new Font("Segoe UI", 16f, FontStyle.Bold),
                ForeColor = Color.FromArgb(0, 230, 153),
                AutoSize = true,
                Location = new Point(20, 15)
            };

            Label lblSubTitle = new Label
            {
                Text = "● AI Model: Qwen 3.5 Omni Flash (Sẵn sàng)",
                Font = new Font("Segoe UI", 9f, FontStyle.Regular),
                ForeColor = Color.FromArgb(120, 90, 255),
                AutoSize = true,
                Location = new Point(210, 22)
            };

            pnlHeader.Controls.Add(lblTitle);
            pnlHeader.Controls.Add(lblSubTitle);
            this.Controls.Add(pnlHeader);

            // Container Panel
            Panel pnlMain = new Panel
            {
                Location = new Point(20, 80),
                Size = new Size(825, 545),
                BackColor = Color.Transparent
            };
            this.Controls.Add(pnlMain);

            // --- Card 1: Trạng thái RAM & Phần cứng ---
            Panel cardHardware = CreateCard(20, 10, 480, 210, "TRẠNG THÁI BỘ NHỚ THỜI GIAN THỰC");

            pnlRamProgress = new Panel
            {
                Location = new Point(20, 45),
                Size = new Size(440, 26),
                BackColor = Color.FromArgb(35, 38, 52)
            };
            pnlRamProgress.Paint += PnlRamProgress_Paint;
            cardHardware.Controls.Add(pnlRamProgress);

            lblRamPercent = new Label
            {
                Text = "0%",
                Font = new Font("Segoe UI", 22f, FontStyle.Bold),
                ForeColor = Color.FromArgb(0, 230, 153),
                AutoSize = true,
                Location = new Point(20, 85)
            };
            cardHardware.Controls.Add(lblRamPercent);

            lblRamDetails = new Label
            {
                Text = "Đang dùng: 0 MB / 0 MB",
                Font = new Font("Segoe UI", 10.5f, FontStyle.Regular),
                ForeColor = Color.Gainsboro,
                AutoSize = true,
                Location = new Point(140, 90)
            };
            cardHardware.Controls.Add(lblRamDetails);

            lblFreeRam = new Label
            {
                Text = "RAM Còn Trống: 0 MB",
                Font = new Font("Segoe UI", 10f, FontStyle.Regular),
                ForeColor = Color.FromArgb(0, 230, 153),
                AutoSize = true,
                Location = new Point(140, 115)
            };
            cardHardware.Controls.Add(lblFreeRam);

            lblTimerRes = new Label
            {
                Text = "⚡ CPU Timer Resolution: 0.5 ms (Độ trễ Input Lag tối thiểu)",
                Font = new Font("Segoe UI", 9f, FontStyle.Italic),
                ForeColor = Color.FromArgb(170, 170, 200),
                AutoSize = true,
                Location = new Point(20, 160)
            };
            cardHardware.Controls.Add(lblTimerRes);

            chkAutoClean = new CheckBox
            {
                Text = "Tự động dọn khi RAM vượt quá 85%",
                Checked = true,
                ForeColor = Color.Yellow,
                AutoSize = true,
                Location = new Point(20, 182)
            };
            chkAutoClean.CheckedChanged += (s, e) => { autoCleanEnabled = chkAutoClean.Checked; };
            cardHardware.Controls.Add(chkAutoClean);

            pnlMain.Controls.Add(cardHardware);

            // --- Card 2: Các nút hành động nhanh ---
            Panel cardActions = CreateCard(520, 10, 290, 210, "HÀNH ĐỘNG NHANH");

            Button btnCleanAll = CreateButton("⚡ Dọn Toàn Diện RAM", 15, 38, 260, 34, Color.FromArgb(0, 160, 100));
            btnCleanAll.Click += (s, e) => CleanAllRam();
            cardActions.Controls.Add(btnCleanAll);

            Button btnStandby = CreateButton("🎯 Xả Standby Cache (Chống Lag)", 15, 76, 260, 32, Color.FromArgb(40, 50, 75));
            btnStandby.Click += (s, e) => PurgeStandby();
            cardActions.Controls.Add(btnStandby);

            Button btnCleanDisk = CreateButton("🧹 Dọn Rác & Temp (%TEMP%)", 15, 112, 260, 32, Color.FromArgb(40, 50, 75));
            btnCleanDisk.Click += (s, e) => CleanDiskTemp();
            cardActions.Controls.Add(btnCleanDisk);

            Button btnShader = CreateButton("🎮 Dọn GPU Shader Cache", 15, 148, 260, 32, Color.FromArgb(40, 50, 75));
            btnShader.Click += (s, e) => CleanGpuShader();
            cardActions.Controls.Add(btnShader);

            pnlMain.Controls.Add(cardActions);

            // --- Card 3: Trợ lý AI & Lịch sử hoạt động ---
            Panel cardAi = CreateCard(20, 235, 790, 300, "TRỢ LÝ TỰ ĐỘNG HÓA AI (QWEN 3.5 OMNI FLASH)");

            rtbLog = new RichTextBox
            {
                Location = new Point(20, 35),
                Size = new Size(750, 190),
                BackColor = Color.FromArgb(12, 13, 18),
                ForeColor = Color.FromArgb(200, 210, 230),
                Font = new Font("Consolas", 9.5f),
                ReadOnly = true,
                BorderStyle = BorderStyle.None
            };
            cardAi.Controls.Add(rtbLog);

            txtAiInput = new TextBox
            {
                Location = new Point(20, 240),
                Size = new Size(600, 30),
                BackColor = Color.FromArgb(30, 33, 46),
                ForeColor = Color.White,
                Font = new Font("Segoe UI", 10.5f),
                BorderStyle = BorderStyle.FixedSingle,
            };
            txtAiInput.KeyDown += (s, e) =>
            {
                if (e.KeyCode == Keys.Enter)
                {
                    e.SuppressKeyPress = true;
                    SendToAi();
                }
            };
            cardAi.Controls.Add(txtAiInput);

            btnAiSend = CreateButton("Gửi Lệnh Cho AI", 630, 238, 140, 32, Color.FromArgb(120, 90, 255));
            btnAiSend.Click += (s, e) => SendToAi();
            cardAi.Controls.Add(btnAiSend);

            pnlMain.Controls.Add(cardAi);

            // Welcome log
            LogMessage("MemScript-AI v1.0 đã sẵn sàng. Gõ câu lệnh bất kỳ (ví dụ: 'Tối ưu máy cho tao chơi game') để AI xử lý!", Color.Cyan);

            // Timer cập nhật thông số mỗi 1 giây
            refreshTimer = new System.Windows.Forms.Timer { Interval = 1000 };
            refreshTimer.Tick += RefreshTimer_Tick;
            refreshTimer.Start();

            // Khay hệ thống
            trayIcon = new NotifyIcon
            {
                Icon = SystemIcons.Application,
                Text = "MemScript-AI Hardware Optimizer",
                Visible = true
            };
            ContextMenu trayMenu = new ContextMenu();
            trayMenu.MenuItems.Add("Mở Giao Diện", (s, e) => { this.Show(); this.WindowState = FormWindowState.Normal; });
            trayMenu.MenuItems.Add("Dọn Nhanh RAM", (s, e) => CleanAllRam());
            trayMenu.MenuItems.Add("-");
            trayMenu.MenuItems.Add("Thoát", (s, e) => { trayIcon.Visible = false; Application.Exit(); });
            trayIcon.ContextMenu = trayMenu;
            trayIcon.DoubleClick += (s, e) => { this.Show(); this.WindowState = FormWindowState.Normal; };
        }

        private void RefreshTimer_Tick(object sender, EventArgs e)
        {
            UpdateMemoryStats();

            // Tự động dọn RAM ngầm nếu vượt ngưỡng
            if (autoCleanEnabled && currentRamLoad >= autoCleanThreshold)
            {
                LogMessage(string.Format("[Auto-Clean] RAM đạt {0}% (Vượt ngưỡng {1}%), đang tự động dọn...", currentRamLoad, autoCleanThreshold), Color.Yellow);
                CleanAllRam();
            }
        }

        private void PnlRamProgress_Paint(object sender, PaintEventArgs e)
        {
            Graphics g = e.Graphics;
            g.SmoothingMode = SmoothingMode.AntiAlias;

            int width = pnlRamProgress.Width;
            int height = pnlRamProgress.Height;

            int fillWidth = (int)((width * currentRamLoad) / 100.0);

            Color progressColor = Color.FromArgb(0, 230, 153);
            if (currentRamLoad > 85) progressColor = Color.FromArgb(255, 75, 75);
            else if (currentRamLoad > 70) progressColor = Color.FromArgb(255, 175, 40);

            using (Brush bgBrush = new SolidBrush(Color.FromArgb(30, 33, 46)))
            {
                g.FillRectangle(bgBrush, 0, 0, width, height);
            }

            if (fillWidth > 0)
            {
                using (Brush fillBrush = new SolidBrush(progressColor))
                {
                    g.FillRectangle(fillBrush, 0, 0, fillWidth, height);
                }
            }
        }

        private void UpdateMemoryStats()
        {
            MEMORYSTATUSEX mem = new MEMORYSTATUSEX();
            mem.dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX));
            if (GlobalMemoryStatusEx(ref mem))
            {
                currentRamLoad = (int)mem.dwMemoryLoad;
                currentTotalMB = mem.ullTotalPhys / (1024 * 1024);
                currentFreeMB = mem.ullAvailPhys / (1024 * 1024);
                currentUsedMB = currentTotalMB - currentFreeMB;

                lblRamPercent.Text = currentRamLoad + "%";
                if (currentRamLoad > 85) lblRamPercent.ForeColor = Color.FromArgb(255, 75, 75);
                else if (currentRamLoad > 70) lblRamPercent.ForeColor = Color.FromArgb(255, 175, 40);
                else lblRamPercent.ForeColor = Color.FromArgb(0, 230, 153);

                lblRamDetails.Text = string.Format("Đang dùng: {0:N0} MB / {1:N0} MB", currentUsedMB, currentTotalMB);
                lblFreeRam.Text = string.Format("RAM Còn Trống: {0:N0} MB", currentFreeMB);

                pnlRamProgress.Invalidate();
            }
        }

        private void CleanAllRam()
        {
            ulong before = currentUsedMB;
            PurgeCommand(MemoryEmptyWorkingSets);
            PurgeCommand(MemoryPurgeStandbyList);
            PurgeCommand(MemoryPurgeLowPriorityStandbyList);
            PurgeCommand(MemoryFlushModifiedList);
            UpdateMemoryStats();
            ulong after = currentUsedMB;
            ulong freed = (before > after) ? (before - after) : 0;

            LogMessage(string.Format("✔ [RAM] Đã dọn toàn diện: {0:N0} MB -> {1:N0} MB (Thu hồi được {2:N0} MB RAM)!", before, after, freed), Color.FromArgb(0, 230, 153));
        }

        private void PurgeStandby()
        {
            PurgeCommand(MemoryPurgeStandbyList);
            UpdateMemoryStats();
            LogMessage("✔ [RAM] Đã xả sạch Standby Cache (Chống giật lag game thành công)!", Color.FromArgb(0, 230, 153));
        }

        private bool PurgeCommand(int cmd)
        {
            GCHandle handle = GCHandle.Alloc(cmd, GCHandleType.Pinned);
            try
            {
                int status = NtSetSystemInformation(SystemMemoryListInformation, handle.AddrOfPinnedObject(), sizeof(int));
                return status >= 0;
            }
            catch { return false; }
            finally { handle.Free(); }
        }

        private void CleanDiskTemp()
        {
            int files = 0;
            long bytes = 0;
            string temp1 = Path.GetTempPath();
            string temp2 = @"C:\Windows\Temp";
            string crashDumps = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "CrashDumps");

            CleanDir(temp1, ref files, ref bytes);
            CleanDir(temp2, ref files, ref bytes);
            CleanDir(crashDumps, ref files, ref bytes);
            SHEmptyRecycleBin(IntPtr.Zero, null, 7);

            long mb = bytes / (1024 * 1024);
            LogMessage(string.Format("✔ [DISK] Đã dọn {0} file rác (%TEMP%, CrashDumps), thu hồi {1} MB ổ cứng!", files, mb), Color.FromArgb(0, 230, 153));
        }

        private void CleanGpuShader()
        {
            int files = 0;
            long bytes = 0;
            string local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);

            CleanDir(Path.Combine(local, "D3DSCache"), ref files, ref bytes);
            CleanDir(Path.Combine(local, @"NVIDIA\DXCache"), ref files, ref bytes);
            CleanDir(Path.Combine(local, @"NVIDIA\GLCache"), ref files, ref bytes);
            CleanDir(Path.Combine(local, @"AMD\DxCache"), ref files, ref bytes);

            long mb = bytes / (1024 * 1024);
            LogMessage(string.Format("✔ [GPU] Đã dọn {0} file Shader Cache, thu hồi {1} MB!", files, mb), Color.FromArgb(0, 230, 153));
        }

        private void CleanDir(string dir, ref int count, ref long bytes)
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

        private void SendToAi()
        {
            string prompt = txtAiInput.Text.Trim();
            if (string.IsNullOrEmpty(prompt)) return;

            txtAiInput.Clear();
            LogMessage(string.Format("\nBạn: {0}", prompt), Color.White);
            LogMessage("🤖 AI đang phân tích yêu cầu...", Color.FromArgb(120, 90, 255));

            btnAiSend.Enabled = false;

            Thread aiThread = new Thread(() =>
            {
                try
                {
                    string sysPrompt = string.Format("Bạn là AI quản lý phần cứng Windows của MemScript-AI. RAM đang dùng: {0}MB/{1}MB ({2}%). Trả lời ngắn gọn 2 câu bằng tiếng Việt và đề xuất tối ưu.", currentUsedMB, currentTotalMB, currentRamLoad);
                    string jsonBody = "{\"model\":\"" + MODEL + "\",\"messages\":[{\"role\":\"system\",\"content\":\"" + sysPrompt + "\"},{\"role\":\"user\",\"content\":\"" + prompt.Replace("\"", "\\\"") + "\"}],\"max_tokens\":256}";

                    using (WebClient client = new WebClient())
                    {
                        client.Encoding = Encoding.UTF8;
                        client.Headers[HttpRequestHeader.ContentType] = "application/json";
                        client.Headers[HttpRequestHeader.Authorization] = "Bearer " + API_KEY;
                        string res = client.UploadString(BASE_URL, jsonBody);

                        int contentIdx = res.IndexOf("\"content\":");
                        if (contentIdx > 0)
                        {
                            int start = res.IndexOf("\"", contentIdx + 10) + 1;
                            int end = res.IndexOf("\"", start);
                            string reply = res.Substring(start, end - start).Replace("\\n", "\n").Replace("\\\"", "\"");

                            this.Invoke((MethodInvoker)delegate
                            {
                                LogMessage(string.Format("AI: {0}", reply), Color.FromArgb(0, 230, 153));

                                // Tự động tối ưu nếu câu hỏi liên quan đến dọn dẹp/chơi game
                                string lower = prompt.ToLower();
                                if (lower.Contains("tối ưu") || lower.Contains("dọn") || lower.Contains("game") || lower.Contains("lag") || lower.Contains("ram"))
                                {
                                    LogMessage("⚡ [AI Auto-Pilot] Đang tự động kích hoạt tối ưu toàn diện hệ thống...", Color.Magenta);
                                    CleanAllRam();
                                    PurgeStandby();
                                    CleanDiskTemp();
                                    CleanGpuShader();
                                    uint cur;
                                    NtSetTimerResolution(5000, true, out cur);
                                    LogMessage("✔ Đã hoàn tất chu trình tối ưu máy tính!", Color.FromArgb(0, 230, 153));
                                }
                            });
                        }
                    }
                }
                catch (Exception ex)
                {
                    this.Invoke((MethodInvoker)delegate
                    {
                        LogMessage(string.Format("✖ [Lỗi AI]: {0}", ex.Message), Color.Red);
                    });
                }
                finally
                {
                    this.Invoke((MethodInvoker)delegate
                    {
                        btnAiSend.Enabled = true;
                    });
                }
            })
            { IsBackground = true };
            aiThread.Start();
        }

        private void LogMessage(string msg, Color color)
        {
            rtbLog.SelectionStart = rtbLog.TextLength;
            rtbLog.SelectionLength = 0;
            rtbLog.SelectionColor = color;
            rtbLog.AppendText(string.Format("[{0:HH:mm:ss}] {1}\n", DateTime.Now, msg));
            rtbLog.ScrollToCaret();
        }

        private Panel CreateCard(int x, int y, int w, int h, string title)
        {
            Panel card = new Panel
            {
                Location = new Point(x, y),
                Size = new Size(w, h),
                BackColor = Color.FromArgb(24, 26, 36),
                Padding = new Padding(15)
            };

            Label lbl = new Label
            {
                Text = title,
                Font = new Font("Segoe UI", 8.5f, FontStyle.Bold),
                ForeColor = Color.FromArgb(140, 145, 170),
                Location = new Point(15, 12),
                AutoSize = true
            };
            card.Controls.Add(lbl);

            return card;
        }

        private Button CreateButton(string text, int x, int y, int w, int h, Color bg)
        {
            Button btn = new Button
            {
                Text = text,
                Location = new Point(x, y),
                Size = new Size(w, h),
                BackColor = bg,
                ForeColor = Color.White,
                FlatStyle = FlatStyle.Flat,
                Font = new Font("Segoe UI", 9.5f, FontStyle.Bold),
                Cursor = Cursors.Hand
            };
            btn.FlatAppearance.BorderSize = 0;
            return btn;
        }
    }
}
