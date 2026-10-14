#pragma once

#include <QPlainTextEdit>

class QPaintEvent;
class QResizeEvent;
class QWidget;

class LineNumberArea;

class CodeEditor : public QPlainTextEdit
{
    Q_OBJECT
public:
    explicit CodeEditor(QWidget* parent = nullptr);
    int lineNumberAreaWidth() const;
    void lineNumberAreaPaintEvent(QPaintEvent* event);

protected:
    void resizeEvent(QResizeEvent* event) override;

private slots:
    void updateLineNumberAreaWidth();
    void updateLineNumberArea(const QRect& rect);
    void highlightCurrentLine();

private:
    QWidget* lineNumberArea_{};
    friend class LineNumberArea;
};

class LineNumberArea : public QWidget
{
public:
    explicit LineNumberArea(CodeEditor* editor) : QWidget(editor), editor_(editor) {}
    QSize sizeHint() const override;
protected:
    void paintEvent(QPaintEvent* event) override;
private:
    CodeEditor* editor_;
};
