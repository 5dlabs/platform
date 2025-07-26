# 📥 Project Intake Directory

This directory is monitored by our automated **Project Intake Pipeline** that processes new project submissions.

## 🚀 How to Submit a New Project

To submit a new project for automated processing, follow these steps:

### 1. Prepare Your Documents

You need **exactly two files**:

- **PRD Document** (`.txt` file): Product Requirements Document
- **Architecture Document** (`.md` file): Detailed architecture and project plan

### 2. File Naming Conventions

- PRD file: `project-name-prd.txt` or `prd.txt`
- Architecture file: `project-name-architecture.md` or `architecture.md`

### 3. Submit Your Project

1. Create a new branch: `git checkout -b intake/your-project-name`
2. Add your files to the `intake/` directory
3. Commit with message: `intake: Add [project-name] for processing`
4. Push and create a PR

## 🤖 What Happens Next

Our automated pipeline will:

✅ **Detect** your submitted files
✅ **Extract** the project name from your documents
✅ **Create** a complete project structure under `projects/`
✅ **Initialize** TaskMaster with the Opus model
✅ **Generate** comprehensive tasks from your PRD
✅ **Analyze** task complexity and expand with subtasks
✅ **Cross-reference** tasks with your architecture document
✅ **Create** individual task files and documentation structure
✅ **Submit** a pull request with the complete project setup

## 📋 Example Structure

After processing, your project will look like this:

```
projects/your-project-name/
├── .taskmaster/
│   ├── docs/
│   │   ├── prd.txt
│   │   └── architecture.md
│   └── tasks.json
└── docs/
    ├── task-1/
    │   └── task.txt
    ├── task-2/
    │   └── task.txt
    └── ...
```

## 🎯 Models Used

- **Task Generation**: claude-opus-4-20250514
- **Complexity Analysis**: claude-4-20250514
- **Cross-referencing**: claude-code-action
- **Documentation**: claude-opus-4-20250514

## ⚠️ Important Notes

- Only users with **write access** can trigger the pipeline
- Files in `intake/` are **automatically removed** after processing
- The project name is **auto-extracted** from your document titles
- All generated tasks follow the **no research limits** policy
- Tasks are **automatically expanded** with subtasks for complexity

## 🔄 Process Status

The GitHub Action will:
- Run on any push to `intake/**`
- Create a detailed PR with processing results
- Provide complete status updates and summaries
- Set up the project ready for development workflow

Ready to submit your project? Just add your files and push! 🚀

<!-- Test trigger for workflow v11 - With validation job and simplified finalization -->