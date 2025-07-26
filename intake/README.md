# 📥 Project Intake Directory

This directory is monitored by our automated **Project Intake Pipeline** that processes new project submissions using TaskMaster AI.

## 🚀 How to Use the Intake Pipeline

To submit a new project for automated processing:

### 1. Prepare Your Documents

You need **exactly two files**:

- **PRD Document** (`.txt` file): Product Requirements Document
- **Architecture Document** (`.md` file): Technical architecture and implementation plan

See `EXAMPLE-prd.txt` and `EXAMPLE-architecture.md` for reference formats.

### 2. File Naming Conventions

- PRD file: `your-project-prd.txt` (or any `.txt` file)
- Architecture file: `your-project-architecture.md` (or any `.md` file)

### 3. Submit Your Files

1. **Replace example files**: Replace `EXAMPLE-prd.txt` and `EXAMPLE-architecture.md` with your documents
2. **Commit changes**: `git add intake/` && `git commit -m "intake: Add [project-name] for processing"`
3. **Push to trigger**: `git push` (or create PR to trigger on a branch)

## 🤖 Automated Processing Pipeline

The workflow automatically:

✅ **Validates** API credentials and tools  
✅ **Detects** your PRD and architecture documents  
✅ **Extracts** project name from document content  
✅ **Sets up** TaskMaster workspace in `intake/.taskmaster/`  
✅ **Generates** comprehensive tasks from your PRD  
✅ **Creates** individual task files in `intake/docs/task-X/`  
✅ **Validates** processing with AI cross-referencing  
✅ **Creates** a pull request with all generated files

## 📁 Generated Structure

After processing, you'll find in the `intake/` directory:

```
intake/
├── .taskmaster/
│   ├── docs/
│   │   ├── prd.txt              # Your PRD
│   │   └── architecture.md      # Your architecture
│   ├── tasks/
│   │   └── tasks.json          # Generated task structure
│   └── config.json             # TaskMaster configuration
└── docs/
    ├── task-1/
    │   └── task.txt            # Individual task details
    ├── task-2/
    │   └── task.txt
    └── ...                     # All generated tasks
```

## ⚙️ Technical Details

### AI Models Used
- **Task Generation**: Claude Sonnet (via TaskMaster)
- **Validation**: Claude Haiku (simple API validation)

### Performance Optimizations
- **No research mode**: Faster task generation
- **Simple validation**: Quick API confirmation vs full AI analysis
- **Direct file processing**: Works entirely within `intake/` directory

### Triggers
- **Push events**: Any changes to `intake/**` files
- **Manual trigger**: Via GitHub Actions UI

## 🔍 Monitoring

Track progress via:
- **GitHub Actions**: View workflow runs and logs
- **Pull Requests**: Review generated project structure
- **Commit history**: See automated commits with results

## 📝 Example Usage

1. **Copy your PRD** into `EXAMPLE-prd.txt`
2. **Copy your architecture** into `EXAMPLE-architecture.md`  
3. **Push changes**: The workflow triggers automatically
4. **Review results**: Check the generated PR with task structure

Ready to process your project? Replace the example files and push! 🚀

<!-- Test trigger for workflow v31 - Clean streamlined workflow test -->