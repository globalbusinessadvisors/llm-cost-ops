# Storyboard: Video 02 - Getting Started with LLM Cost Ops

**Video ID**: LLMCO-V02-SETUP
**Storyboard Version**: 1.0.0
**Total Duration**: 19:00
**Last Updated**: 2025-11-16

---

## Scene 1: Opening & Desktop Setup
**Duration**: 0:00 - 1:00 (60 seconds)
**Purpose**: Set expectations, show clean starting point

### Visual Composition
- **Shot Type**: WS - Full desktop view
- **Layout**: Clean macOS or Linux desktop
- **Background**: Professional wallpaper (solid or minimal pattern)

### Screen Elements

**Initial State (0:00-0:10)**
- Clean desktop, no clutter
- Dock/taskbar visible with essential apps
- Date/time visible in corner
- No personal information

**Title Overlay (0:10-0:20)**
- Text fades in: "Getting Started with LLM Cost Ops"
- Font: Inter Bold, 56pt, White with subtle drop shadow
- Position: Center screen, slight vertical offset up
- Background: Semi-transparent dark overlay (20% opacity)

**Agenda Checklist (0:20-0:50)**
- Checklist slides in from right
- Position: Right third of screen
- Background: White card with shadow
- Items appear sequentially (0.3s stagger):
  - ☐ Installation
  - ☐ Configuration
  - ☐ First Project
  - ☐ Track Requests
- Font: Inter Medium, 24pt

**Desktop Preparation (0:50-1:00)**
- Title and checklist fade out
- Cursor moves to Applications/Terminal
- Terminal icon highlights
- Ready to start

### Animation Sequence
1. **0:00-0:10**: Desktop appears (fade from black)
2. **0:10-0:15**: Title fades in
3. **0:15-0:20**: Title settles with slight scale (1.05 → 1.0)
4. **0:20-0:35**: Checklist slides in (from right, 300px)
5. **0:35-0:50**: Checklist items appear (staggered)
6. **0:50-0:55**: Overlays fade out
7. **0:55-1:00**: Cursor moves to Terminal

### Audio Integration
- **Music**: Energetic startup theme
- **Sound Effects**:
  - 0:10: Whoosh (title appears)
  - 0:20: Slide (checklist)
  - 0:35-0:50: Pops (checklist items)
  - 0:55: Click (terminal hover)
- **Narration**: Starts at 0:05, welcomes viewers

---

## Scene 2: Terminal - Checking Requirements
**Duration**: 1:00 - 2:00 (60 seconds)
**Purpose**: Verify system prerequisites

### Visual Composition
- **Shot Type**: MS - Terminal takes 70% of screen
- **Layout**: Terminal center, reference panel on right
- **Terminal**: Dark theme (Dracula or One Dark)

### Screen Elements

**Terminal Window**
- Position: Left 70% of screen
- Size: 1344x1080 (maintains aspect)
- Theme: Dark background (#282C34), light text (#ABB2BF)
- Font: JetBrains Mono, 16pt
- Prompt visible: `user@machine ~ %`

**Reference Panel (Right 30%)**
- Background: Semi-transparent white (#FFFFFF at 95%)
- Title: "System Requirements"
- Content:
  ```
  Required:
  ✓ Docker 20.0+
  ✓ Docker Compose 2.0+
  ✓ 4GB RAM available
  ✓ 10GB disk space

  Optional:
  • Node.js 18+ (for SDK)
  • Python 3.9+ (for SDK)
  ```
- Each checkmark appears as command succeeds

### Command Sequence

**Command 1: Check Docker**
```bash
$ docker --version
Docker version 24.0.6, build ed223bc
```
- Typing effect: 40 characters/second
- Output appears immediately
- Checkmark in reference panel turns green
- Pause: 0.8 seconds

**Command 2: Check Docker Compose**
```bash
$ docker compose version
Docker Compose version v2.23.0
```
- Same typing effect
- Checkmark turns green
- Pause: 0.8 seconds

**Command 3: Check Node.js (Optional)**
```bash
$ node --version
v20.10.0
```
- Typing, output, green check
- Pause: 0.8 seconds

**Command 4: Check Python (Optional)**
```bash
$ python --version
Python 3.11.5
```
- Final command
- All checks green
- Reference panel pulses briefly (success)

### Animation Sequence
1. **1:00-1:05**: Terminal window opens (scale from center)
2. **1:05-1:10**: Reference panel slides in from right
3. **1:10-1:25**: Docker version command (type, execute, result)
4. **1:25-1:40**: Docker Compose command
5. **1:40-1:50**: Node.js command
6. **1:50-2:00**: Python command and success pulse

### Visual Enhancements
- **Cursor**: Block cursor, blinking (0.5s interval)
- **Command Highlight**: Subtle background (#3E4451) behind command
- **Output Color**: Different color (#98C379 for success messages)
- **Checkmarks**: Animated ✗ → ✓ transition (rotation + color change)

### Audio Integration
- **Music**: Minimal, technical background
- **Sound Effects**:
  - Terminal open: Whoosh
  - Typing: Subtle keyboard clicks (not for every character)
  - Command execute: Return key sound
  - Success checkmark: Soft "ding"
  - Final pulse: Triumphant beep
- **Narration**: Explains what each command checks

---

## Scene 3: Docker Installation
**Duration**: 2:00 - 5:00 (180 seconds)
**Purpose**: Install via Docker Compose

### Visual Composition
- **Shot Type**: Split Screen - Terminal left, File Explorer right
- **Layout**: 50/50 split or 60/40 (terminal focus)

### Screen Elements

**Phase 1: Clone Repository (2:00-2:30)**

**Terminal (Left)**
```bash
$ git clone https://github.com/llm-cost-ops/llm-cost-ops.git
Cloning into 'llm-cost-ops'...
remote: Enumerating objects: 1547, done.
remote: Counting objects: 100% (1547/1547), done.
remote: Compressing objects: 100% (892/892), done.
remote: Total 1547 (delta 645), reused 1547 (delta 645)
Receiving objects: 100% (1547/1547), 2.45 MiB | 3.21 MiB/s, done.
Resolving deltas: 100% (645/645), done.

$ cd llm-cost-ops
```

**File Explorer (Right)**
- Shows folder structure appearing as clone progresses
- Highlights key files as they appear:
  - `docker-compose.yml`
  - `.env.example`
  - `README.md`

**Phase 2: Environment Setup (2:30-3:30)**

**Terminal**
```bash
$ cp .env.example .env
$ nano .env    # or vim/code, show editor preference
```

**Split Transitions to: Terminal (left) + Text Editor (right)**

**Text Editor** showing `.env` file:
```bash
# Database Configuration
DATABASE_URL=postgresql://postgres:postgres@db:5432/llm_cost_ops

# API Configuration
API_PORT=8081
WEB_PORT=8080

# Secret Key (generate random)
SECRET_KEY=change-this-to-random-string

# Redis Configuration
REDIS_URL=redis://redis:6379
```

**Callout appears**: "Generate a secure secret key!"
**Command shown**:
```bash
$ openssl rand -hex 32
a8f5f167f44f4964e6c998dee827110c
```

**Editor updates**: SECRET_KEY with generated value

**Phase 3: Start Services (3:30-5:00)**

**Back to full-width terminal**
```bash
$ docker compose up -d
[+] Running 14/14
 ✔ db 6 layers [████████████] 100%     Pulled      12.3s
 ✔ redis 5 layers [████████████] 100%   Pulled      8.7s
 ✔ api 8 layers [████████████] 100%     Pulled      15.2s
[+] Running 3/3
 ✔ Network llm-cost-ops_default    Created         0.1s
 ✔ Container llm-cost-ops-db-1     Started         0.8s
 ✔ Container llm-cost-ops-redis-1  Started         0.9s
 ✔ Container llm-cost-ops-api-1    Started         1.2s

$ docker compose logs -f api
llm-cost-ops-api-1 | Running database migrations...
llm-cost-ops-api-1 | ✓ Migration 001_initial_schema applied
llm-cost-ops-api-1 | ✓ Migration 002_add_budgets applied
llm-cost-ops-api-1 | ✓ Migration 003_add_tags applied
llm-cost-ops-api-1 | Database migrations complete!
llm-cost-ops-api-1 |
llm-cost-ops-api-1 | Starting API server...
llm-cost-ops-api-1 | ✓ API server listening on :8081
llm-cost-ops-api-1 | ✓ Web server listening on :8080
llm-cost-ops-api-1 |
llm-cost-ops-api-1 | LLM Cost Ops ready! 🚀
```

**Callout overlay**: "Ready! Visit http://localhost:8080"

### Animation Sequence
1. **2:00-2:10**: Git clone command types
2. **2:10-2:25**: Clone progress (animated progress bars)
3. **2:25-2:30**: File explorer updates in real-time
4. **2:30-2:35**: Copy .env command
5. **2:35-2:45**: Editor opens with split screen
6. **2:45-3:10**: Edit .env file (highlighted changes)
7. **3:10-3:20**: Generate secret key
8. **3:20-3:30**: Paste into editor, save
9. **3:30-3:35**: Docker compose up command
10. **3:35-3:50**: Pull progress bars (animated)
11. **3:50-4:10**: Container creation checkmarks
12. **4:10-4:50**: Log stream (auto-scrolling)
13. **4:50-5:00**: Success callout appears

### Visual Enhancements
- **Progress Bars**: Animated fill from left to right
- **Checkmarks**: Green ✓ with brief scale animation
- **Logs**: Color-coded (info: blue, success: green)
- **Rocket Emoji**: Animated launch (scale + slight rotation)
- **Callout**: Slide in from bottom with subtle bounce

---

## Scene 4: Browser - First Login
**Duration**: 5:00 - 7:00 (120 seconds)
**Purpose**: Create account, explore dashboard

### Visual Composition
- **Shot Type**: SR - Screen recording of browser
- **Layout**: Full browser window, cropped to content area
- **Browser**: Chrome or Firefox, clean profile

### Screen Elements

**Phase 1: Navigate to App (5:00-5:15)**
- Browser address bar
- Type URL: `http://localhost:8080`
- URL highlights as entered
- Press Enter
- Loading indicator
- Page loads

**Phase 2: Welcome Screen (5:15-5:30)**
- Full page: LLM Cost Ops welcome screen
- Elements:
  - Logo (top center)
  - Tagline: "Cost Operations Platform for LLMs"
  - Two buttons:
    - "Create Account" (primary, blue)
    - "Sign In" (secondary, outline)
- Clean, modern design
- Subtle gradient background

**Phase 3: Create Account (5:30-6:15)**
- Click "Create Account" (button highlights on hover)
- Form slides in:
  ```
  Create Your Account

  Email Address: [demo@example.com        ]
  Password:      [••••••••                ]
  Confirm:       [••••••••                ]

  [Create Account] [Cancel]
  ```
- Form fill animation (typing effect, 50 chars/sec)
- Password strength indicator appears (green: strong)
- Click "Create Account"
- Button shows loading spinner
- Success! Redirect with fade transition

**Phase 4: Empty Dashboard (6:15-7:00)**
- Dashboard loads (fade in)
- Empty state illustration:
  - Center: Illustration of empty graph
  - Text: "No data yet"
  - Subtext: "Create a project to start tracking costs"
  - Button: "Create Your First Project" (pulsing gently)
- Sidebar visible with nav items:
  - Dashboard (active)
  - Projects
  - Analytics
  - Budgets
  - Settings

**Callout appears**: "Let's create your first project!"

### Animation Sequence
1. **5:00-5:05**: Browser opens (fade in)
2. **5:05-5:10**: Type URL (with cursor)
3. **5:10-5:15**: Page loads (loading spinner)
4. **5:15-5:20**: Welcome screen fades in
5. **5:20-5:30**: Buttons have hover states (subtle shadow grow)
6. **5:30-5:35**: Click animation (button press)
7. **5:35-5:40**: Form slides up from bottom
8. **5:40-6:05**: Form fields fill (typing animation)
9. **6:05-6:10**: Password strength bar grows
10. **6:10-6:15**: Submit button clicked, spinner
11. **6:15-6:25**: Redirect fade transition
12. **6:25-6:40**: Dashboard elements appear sequentially
13. **6:40-6:50**: Empty state illustration fades in
14. **6:50-7:00**: CTA button pulses, callout appears

### Visual Enhancements
- **Cursor**: Enhanced visibility (ring or glow)
- **Hover States**: Smooth transitions (0.2s)
- **Form Validation**: Real-time (checkmarks as typing)
- **Loading States**: Professional spinners
- **Empty State**: Friendly illustration (not just text)
- **Callouts**: Speech bubble style, pointing to button

### Audio Integration
- **Music**: Optimistic, friendly
- **Sound Effects**:
  - Page load: Soft whoosh
  - Button hover: Subtle tone shift
  - Button click: Satisfying click
  - Form submit: Send sound
  - Success: Cheerful "ding"
  - Dashboard load: Transition whoosh
- **Narration**: Guide user through each step

---

## Scenes 5-11: Continued Workflow

### Scene 5: Create Project (7:00-8:30)
- Project creation modal
- Form completion
- API key generation and display
- Copy-to-clipboard animation

### Scene 6: SDK Installation - Python (8:30-10:30)
- Terminal returns (split with browser)
- Virtual environment creation
- pip install
- Write first Python script

### Scene 7: Run Python Example (10:30-12:00)
- Execute Python script
- Terminal output showing success
- Browser refresh showing first data point

### Scene 8: Dashboard Data View (12:00-13:30)
- Click through request details
- Explore cost breakdown
- Filter and search demonstrations

### Scene 9: TypeScript Quick Demo (13:30-15:30)
- New terminal window
- npm project init
- Quick TypeScript example
- Another data point in dashboard

### Scene 10: Dashboard Tour (15:30-17:30)
- Navigate through sections
- Show analytics tab
- Preview budgets tab
- Settings overview

### Scene 11: Recap & Next Steps (17:30-19:00)
- Return to checklist from Scene 1
- All items checked off (animated)
- Preview of next video
- Encouraging closing

---

## Production Specifications

### Screen Recording Settings
- **Resolution**: 1920x1080
- **Frame Rate**: 60fps (for smooth typing animations)
- **Cursor**: macOS pointer with enhanced visibility
- **Window Focus**: Automatic hide/show of irrelevant elements

### Browser Configuration
- **Extensions**: All disabled (clean interface)
- **Zoom**: 100% (no scaling)
- **DevTools**: Closed
- **Bookmarks**: Hidden
- **Theme**: Default light or dark (consistent)

### Terminal Configuration
- **Theme**: One Dark or Dracula
- **Font**: JetBrains Mono, 16pt
- **Cursor**: Block, blinking
- **Prompt**: Simple (username@machine dir %)
- **Colors**: Standard ANSI colors

### Editor Configuration
- **Editor**: VS Code
- **Theme**: One Dark Pro
- **Font**: JetBrains Mono, 16pt
- **Mini map**: Hidden
- **Sidebar**: Collapsed when not needed
- **Zen Mode**: Consider for focus

---

**Storyboard Version**: 1.0.0
**Created**: 2025-11-16
**Total Scenes**: 11
**Recording Time Estimate**: 2 days (with retakes)
