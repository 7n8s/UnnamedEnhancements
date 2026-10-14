#include "MainWindow.h"
#include "CodeEditor.h"
#include "LuauHighlighter.h"
#include "LuauRunner.h"

#include <QAction>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QFontDatabase>
#include <QHBoxLayout>
#include <QLabel>
#include <QMessageBox>
#include <QPlainTextEdit>
#include <QSplitter>
#include <QTabWidget>
#include <QTextDocument>
#include <QToolButton>
#include <QVBoxLayout>
#include <QMenuBar>
#include <QKeySequence>

namespace {
const char* starter = "local greeting = \"hello from unnamed\"\nprint(greeting)\n\nfor i = 1, 3 do\n    print(\"count:\", i)\nend";
}

MainWindow::MainWindow(QWidget* parent) : QMainWindow(parent)
{
    resize(1180, 790);
    setMinimumSize(820, 560);
    setWindowTitle("Unnamed Executor");
    auto* root = new QWidget(this);
    auto* layout = new QVBoxLayout(root);
    layout->setContentsMargins(14, 12, 14, 12);
    layout->setSpacing(10);

    auto* header = new QWidget(root);
    auto* headerLayout = new QHBoxLayout(header);
    headerLayout->setContentsMargins(0, 0, 0, 0);
    auto* brand = new QLabel("UNNAMED <span style='color:#a99aff'>/</span> LUUAU", header);
    brand->setObjectName("brand");
    headerLayout->addWidget(brand);
    auto* subtitle = new QLabel("STANDALONE SCRIPT WORKSPACE", header);
    subtitle->setObjectName("subtitle");
    headerLayout->addSpacing(14);
    headerLayout->addWidget(subtitle);
    headerLayout->addStretch();

    auto* newButton = new QToolButton(header);
    newButton->setText("+ New");
    connect(newButton, &QToolButton::clicked, this, [this] { newTab(); });
    headerLayout->addWidget(newButton);

    auto* runButton = new QToolButton(header);
    runButton->setText("▶  Run");
    runButton->setObjectName("runButton");
    runButton->setCursor(Qt::PointingHandCursor);
    connect(runButton, &QToolButton::clicked, this, &MainWindow::runCurrent);
    headerLayout->addWidget(runButton);
    layout->addWidget(header);

    auto* splitter = new QSplitter(Qt::Vertical, root);
    tabs_ = new QTabWidget(splitter);
    tabs_->setTabsClosable(true);
    tabs_->setMovable(true);
    tabs_->setDocumentMode(true);
    connect(tabs_, &QTabWidget::tabCloseRequested, this, &MainWindow::closeTab);

    output_ = new QPlainTextEdit(splitter);
    output_->setObjectName("output");
    output_->setReadOnly(true);
    output_->setPlaceholderText("Execution output will appear here.");
    output_->setMaximumBlockCount(2500);
    splitter->addWidget(tabs_);
    splitter->addWidget(output_);
    splitter->setStretchFactor(0, 4);
    splitter->setStretchFactor(1, 1);
    layout->addWidget(splitter, 1);

    status_ = new QLabel("Ready  •  Standalone Luau sandbox", root);
    status_->setObjectName("status");
    layout->addWidget(status_);
    setCentralWidget(root);

    auto* fileMenu = menuBar()->addMenu("&File");
    auto* newAction = fileMenu->addAction("&New tab");
    newAction->setShortcut(QKeySequence::AddTab);
    connect(newAction, &QAction::triggered, this, [this] { newTab(); });
    auto* openAction = fileMenu->addAction("&Open…");
    openAction->setShortcut(QKeySequence::Open);
    connect(openAction, &QAction::triggered, this, &MainWindow::openFile);
    auto* saveAction = fileMenu->addAction("&Save");
    saveAction->setShortcut(QKeySequence::Save);
    connect(saveAction, &QAction::triggered, this, [this] { saveTab(tabs_->currentIndex()); });
    auto* saveAsAction = fileMenu->addAction("Save &As…");
    connect(saveAsAction, &QAction::triggered, this, [this] { saveTab(tabs_->currentIndex(), true); });
    auto* runAction = fileMenu->addAction("&Run current tab");
    runAction->setShortcut(QKeySequence(Qt::CTRL | Qt::Key_Return));
    connect(runAction, &QAction::triggered, this, &MainWindow::runCurrent);

    auto* outputMenu = menuBar()->addMenu("&Output");
    auto* clearAction = outputMenu->addAction("&Clear output");
    connect(clearAction, &QAction::triggered, output_, &QPlainTextEdit::clear);

    setStyleSheet(R"(
        QMainWindow, QWidget { background:#101114; color:#e7e6ee; }
        QMenuBar { background:#15161a; color:#d7d7df; padding:5px; }
        QMenuBar::item { padding:6px 10px; }
        QMenuBar::item:selected, QMenu::item:selected { background:#292633; }
        QMenu { background:#191a20; border:1px solid #33343d; }
        QMenu::item { padding:7px 24px; }
        #brand { color:#f0eff7; font-size:15px; font-weight:700; letter-spacing:1px; }
        #subtitle { color:#777987; font-size:10px; letter-spacing:1px; }
        QToolButton { background:#22232a; color:#e7e6ee; border:1px solid #34353e;
                      border-radius:7px; padding:8px 13px; }
        QToolButton:hover { background:#30313a; }
        #runButton { background:#9a88ff; color:#151020; font-weight:700; border:0; padding:9px 17px; }
        #runButton:hover { background:#b0a3ff; }
        QTabWidget::pane { border:1px solid #292a32; border-radius:8px; top:-1px; }
        QTabBar::tab { background:#17181d; color:#8f90a0; padding:10px 15px;
                       border:1px solid #292a32; border-bottom:0; margin-right:3px; }
        QTabBar::tab:selected { background:#20212a; color:#e8e5ff; }
        QPlainTextEdit { background:#15161a; color:#e3e2eb; border:1px solid #292a32;
                         border-radius:8px; padding:12px; selection-background-color:#4d427e; }
        #output { background:#111216; color:#bcb8d2; }
        #status { color:#858593; font-size:11px; padding-left:3px; }
        QSplitter::handle { background:#24252c; height:5px; }
    )");
    newTab(QString::fromUtf8(starter), "example.luau");
}

CodeEditor* MainWindow::currentEditor() const
{
    return qobject_cast<CodeEditor*>(tabs_->currentWidget());
}

void MainWindow::newTab(const QString& text, const QString& title)
{
    auto* editor = new CodeEditor(tabs_);
    editor->setObjectName("editor");
    editor->setFont(QFontDatabase::systemFont(QFontDatabase::FixedFont));
    editor->setTabStopDistance(4 * QFontMetrics(editor->font()).horizontalAdvance(' '));
    editor->setPlaceholderText("-- write Luau here");
    editor->setPlainText(text);
    new LuauHighlighter(editor->document());
    const int index = tabs_->addTab(editor, title);
    tabs_->setCurrentIndex(index);
    connect(editor->document(), &QTextDocument::modificationChanged, this,
            [this, editor](bool) {
                const int index = tabs_->indexOf(editor);
                if (index >= 0) updateTabTitle(index);
            });
    editor->document()->setModified(false);
    editor->setFocus();
}

void MainWindow::closeTab(int index)
{
    auto* editor = qobject_cast<CodeEditor*>(tabs_->widget(index));
    if (!editor) return;
    if (editor->document()->isModified())
    {
        const auto answer = QMessageBox::question(this, "Unsaved changes",
            "Save changes before closing this tab?",
            QMessageBox::Save | QMessageBox::Discard | QMessageBox::Cancel);
        if (answer == QMessageBox::Cancel) return;
        if (answer == QMessageBox::Save && !saveTab(index)) return;
    }
    QWidget* page = tabs_->widget(index);
    tabs_->removeTab(index);
    page->deleteLater();
    if (tabs_->count() == 0) newTab();
}

void MainWindow::updateTabTitle(int index)
{
    auto* editor = qobject_cast<CodeEditor*>(tabs_->widget(index));
    if (!editor) return;
    QString title = tabs_->tabText(index);
    if (title.startsWith("*")) title.remove(0, 1);
    if (editor->document()->isModified()) title.prepend("*");
    tabs_->setTabText(index, title);
}

bool MainWindow::saveTab(int index, bool saveAs)
{
    auto* editor = qobject_cast<CodeEditor*>(tabs_->widget(index));
    if (!editor) return false;
    QString path = editor->property("filePath").toString();
    if (saveAs || path.isEmpty())
        path = QFileDialog::getSaveFileName(this, "Save Luau script", path,
            "Luau scripts (*.luau *.lua);;All files (*)");
    if (path.isEmpty()) return false;
    if (QFileInfo(path).suffix().isEmpty()) path += ".luau";

    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text))
    {
        QMessageBox::warning(this, "Save failed", file.errorString());
        return false;
    }
    file.write(editor->toPlainText().toUtf8());
    editor->setProperty("filePath", path);
    editor->document()->setModified(false);
    const int actualIndex = tabs_->indexOf(editor);
    if (actualIndex >= 0) {
        tabs_->setTabText(actualIndex, QFileInfo(path).fileName());
        updateTabTitle(actualIndex);
    }
    setStatus("Saved " + QFileInfo(path).fileName());
    return true;
}

void MainWindow::openFile()
{
    const QString path = QFileDialog::getOpenFileName(this, "Open Luau script", {},
        "Luau scripts (*.luau *.lua);;All files (*)");
    if (path.isEmpty()) return;
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text))
    {
        QMessageBox::warning(this, "Open failed", file.errorString());
        return;
    }
    newTab(QString::fromUtf8(file.readAll()), QFileInfo(path).fileName());
    auto* editor = currentEditor();
    editor->setProperty("filePath", path);
    editor->document()->setModified(false);
    setStatus("Opened " + QFileInfo(path).fileName());
}

void MainWindow::runCurrent()
{
    auto* editor = currentEditor();
    if (!editor) return;
    output_->clear();
    output_->appendPlainText("> Running " + tabs_->tabText(tabs_->currentIndex()).remove('*') + " …");
    output_->appendPlainText(LuauRunner::execute(editor->toPlainText()));
    setStatus("Execution finished");
}

void MainWindow::setStatus(const QString& text)
{
    status_->setText(text + "  •  Standalone Luau sandbox");
}
