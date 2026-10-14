#pragma once
#include <QMainWindow>

class QTabWidget;
class QPlainTextEdit;
class QLabel;
class CodeEditor;

class MainWindow : public QMainWindow
{
    Q_OBJECT
public:
    explicit MainWindow(QWidget* parent = nullptr);
private:
    CodeEditor* currentEditor() const;
    void newTab(const QString& text = QString(), const QString& title = "untitled.luau");
    void closeTab(int index);
    void updateTabTitle(int index);
    bool saveTab(int index, bool saveAs = false);
    void openFile();
    void runCurrent();
    void setStatus(const QString& text);

    QTabWidget* tabs_{};
    QPlainTextEdit* output_{};
    QLabel* status_{};
};
