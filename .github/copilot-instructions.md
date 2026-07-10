# GitHub Copilot Instructions  <!-- Quick reference router - directs to detailed task-specific guides based on user intent -->

> **⚠️ IMPORTANT:** This file is automatically read by GitHub Copilot.
> It routes you to the right task-specific guide.

---

## 🎯 Task Router

### **When asked about Project Context:**
**👉 READ:** `AGENTS.md` (project overview, tech stack, structure)
**Use case:** Understanding architecture, available commands, configuration

### **When asked to update AGENTS.md:**
**👉 READ:** `.github/templates/agents.md` (structure template)
**Action:** Keep synchronized with actual code changes

---

## 📚 Quick File Reference

- **Project Overview:** `AGENTS.md`

---

## 🚫 Critical Rules (ALWAYS Follow)

- ❌ **NEVER** skip reading the task-specific guide
- ❌ **NEVER** use placeholder data or guess
- ✅ **ALWAYS** read the appropriate guide first
- ✅ **ALWAYS** validate results against quality criteria
- ✅ **ALWAYS** document verification commands with dates

---

## ⚡ Quick Examples

tbd

---

## 🔄 Workflow Pattern

```
User Request → copilot-instructions.md (routing)
              ↓
Task-Specific Guide (details + workflow)
              ↓
Execute & Validate
```

---

## 📝 Remember

- **Accuracy > Speed** (especially for security)
- **Read the guide** (don't rely on memory)
- **Document everything** (commands, dates, results)
- **Test your changes** (`mvn verify`)

---

**Version:** 11.0
**Last Updated:** 2026-06-11
**Maintained by:** Markus Sprunck

**Changelog:**
- v1.0 (2026-07-10): First draft
