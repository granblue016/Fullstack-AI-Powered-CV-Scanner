# 🤖 AI Resume Scanner

> AI-powered CV analysis system with intelligent scoring, cover letter generation, and email automation.

[![Rust](https://img.shields.io/badge/Rust-Backend-orange?logo=rust)](https://www.rust-lang.org/)
[![Astro](https://img.shields.io/badge/Astro-Frontend-purple?logo=astro)](https://astro.build/)
[![React](https://img.shields.io/badge/React-UI-blue?logo=react)](https://react.dev/)
[![Gemini](https://img.shields.io/badge/AI-Gemini-green?logo=google)](https://ai.google.dev/)

## ✨ Features

- **📊 Intelligent CV Scoring**: 4-criteria analysis (Technical Skills, Experience, Culture Fit, Growth Potential)
- **🇻🇳 Vietnamese Support**: Full Vietnamese explanations and bilingual interface
- **✉️ Email Generation**: 5 types of professional emails (Application, Follow-up, Offer Response, Negotiation, Decline)
- **📝 Cover Letter**: Auto-generated personalized cover letters
- **📄 Multi-format**: Supports PDF and DOCX parsing
- **🎯 Skill Matching**: Advanced skill extraction with years of experience
- **🏆 Certificate Detection**: Automatic certification matching
- **⚡ Real-time Analysis**: Instant AI-powered insights

## 🚀 Quick Start

### Prerequisites

- **Node.js** v18+ ([Download](https://nodejs.org/))
- **Rust** latest stable ([Install](https://rustup.rs/))
- **Gemini API Key** ([Get Free Key](https://aistudio.google.com/app/apikey))

### Installation

```powershell
# 1. Clone the repository
git clone https://github.com/d0ngle8k/Fullstack-AI-Powered-CV-Scanner.git
cd Fullstack-AI-Powered-CV-Scanner

# 2. Install frontend dependencies
cd frontend
npm install
cd ..

# 3. Configure backend environment
# Create backend/.env with:
# GEMINI_API_KEY=your-api-key-here
# PORT=3001
```

### Run Application

#### Option 1: Production Mode (Recommended)
```powershell
.\start.ps1
```
- Runs both services in background
- Auto-monitoring with combined logs
- Press **Ctrl+C** to stop all services

#### Option 2: Development Mode
```powershell
.\dev.ps1
```
- Backend in separate window
- Frontend in current window
- Easier debugging with separate logs

#### Option 3: Manual Start
```powershell
# Terminal 1 - Backend
cd backend
cargo run

# Terminal 2 - Frontend  
cd frontend
npm run dev
```

### Stop Services

```powershell
.\stop.ps1
```

Or press **Ctrl+C** in the running terminal.

## 🌐 Access Points

- **Frontend**: http://localhost:4321
- **Backend API**: http://localhost:3001

## 📡 API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/analyze` | POST | Analyze CV against JD |
| `/api/cover-letter` | POST | Generate cover letter |
| `/api/email-reply` | POST | Generate email reply |
| `/api/documents/export` | POST | Export to DOCX/PDF/HTML |

## 🏗️ Project Structure

```
Fullstack-AI-Powered-CV-Scanner/
├── backend/              # Rust API server
│   ├── src/
│   │   ├── main.rs      # Axum web server
│   │   ├── models.rs    # Data structures
│   │   ├── ai/          # AI modules (Gemini integration)
│   │   ├── parser/      # PDF/DOCX parsing
│   │   └── documents/   # Document export
│   └── Cargo.toml
│
├── frontend/             # Astro + React UI
│   ├── src/
│   │   ├── pages/       # Astro pages
│   │   └── components/  # React components
│   └── package.json
│
├── shared/               # TypeScript types
│   └── types.ts
│
├── start.ps1            # Start both services
├── stop.ps1             # Stop all services
├── dev.ps1              # Development mode
└── README.md
```

## 🎯 Scoring Algorithm

### 4-Criteria Breakdown (0-10 points each)

1. **Technical Match** - Percentage of required skills matched
2. **Experience Level** - Ghost skills detection (leadership, project management)
3. **Culture Fit** - Red flags analysis (job hopping, employment gaps)
4. **Growth Potential** - Learning ability and skill diversity

**Overall Score** = `((Tech + Exp + Culture + Growth) / 4) × 10` → **0-100%**

### Verdict Classification

- **75-100%**: HIRE ✅ - Top candidate
- **50-74%**: MAYBE ⚠️ - Needs consideration
- **0-49%**: PASS ❌ - Not suitable

## 🔧 Configuration

### Backend (.env)
```env
GEMINI_API_KEY=your-gemini-api-key
PORT=3001
RUST_LOG=info
```

### Frontend
Configure in `frontend/astro.config.mjs` (default port: 4321)

## 🛠️ Development

### Backend Commands
```powershell
cd backend

# Run development server
cargo run

# Build release
cargo build --release

# Run tests
cargo test
```

### Frontend Commands
```powershell
cd frontend

# Development server
npm run dev

# Build for production
npm run build

# Preview production
npm run preview
```

## 📦 Tech Stack

### Backend
- **Rust** - Performance and safety
- **Axum** - Modern web framework
- **Gemini AI** - Intelligent analysis
- **pdf-extract** - PDF parsing
- **docx-rs** - DOCX generation

### Frontend
- **Astro 4** - Static site generator
- **React 18** - UI components
- **TanStack Query** - Data fetching
- **Tailwind CSS** - Styling
- **Lucide React** - Icons

## 🔒 Security

- API keys stored in `.env` (gitignored)
- No sensitive data in commits
- CORS protection enabled
- Input validation on all endpoints

## 🐛 Troubleshooting

### Backend won't start
```powershell
# Check if .env exists
Test-Path backend\.env

# Verify Rust installation
cargo --version

# Clean and rebuild
cd backend
cargo clean
cargo build
```

### Frontend won't start
```powershell
# Reinstall dependencies
cd frontend
Remove-Item node_modules -Recurse -Force
npm install

# Check Node version
node --version  # Should be 18+
```

### Port already in use
```powershell
# Stop existing services
.\stop.ps1

# Or manually kill processes
Get-Process | Where-Object {$_.ProcessName -match "resume-scanner|node|astro"} | Stop-Process -Force
```

## 📝 License

MIT License - feel free to use this project for your own purposes.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📧 Contact

For questions or support, please open an issue on GitHub.

---

**Built with ⚡ Rust, 🤖 AI, and 🎨 Modern Web Technologies**
# Kill processes on port 3001 (backend)
netstat -ano | findstr :3001
taskkill /PID <PID> /F

# Kill processes on port 4321 (frontend)
netstat -ano | findstr :4321
taskkill /PID <PID> /F
```

## 📚 Documentation

- **Core NLP Module**: `docs/CORE_NLP_MODULE.md`
- **Implementation**: `docs/CORE_NLP_IMPLEMENTATION.md`
- **Testing Guide**: `docs/CORE_NLP_TESTING_GUIDE.md`

## 🤝 Contributing

1. Never commit `.env` files
2. Never commit `node_modules/` or `target/`
3. Test before committing
4. Keep API keys secure

## 📄 License

Internal project - Not for public distribution
