#pragma once
#include <QSyntaxHighlighter>
#include <QTextCharFormat>

class LuauHighlighter : public QSyntaxHighlighter
{
    Q_OBJECT
public:
    explicit LuauHighlighter(QTextDocument* parent = nullptr);
protected:
    void highlightBlock(const QString& text) override;
private:
    QTextCharFormat keyword_, string_, comment_, number_, builtin_, operator_;
};
