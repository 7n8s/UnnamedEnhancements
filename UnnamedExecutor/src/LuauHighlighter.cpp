#include "LuauHighlighter.h"

#include <QRegularExpression>
#include <QTextDocument>

LuauHighlighter::LuauHighlighter(QTextDocument* parent) : QSyntaxHighlighter(parent)
{
    keyword_.setForeground(QColor("#c6a7ff"));
    keyword_.setFontWeight(QFont::DemiBold);
    string_.setForeground(QColor("#a8d7a0"));
    comment_.setForeground(QColor("#777b88"));
    comment_.setFontItalic(true);
    number_.setForeground(QColor("#e7bd83"));
    builtin_.setForeground(QColor("#82c9e8"));
    operator_.setForeground(QColor("#d4d2df"));
}

void LuauHighlighter::highlightBlock(const QString& text)
{
    static const QStringList keywords = {
        "and","break","do","else","elseif","end","false","for","function","if",
        "in","local","nil","not","or","repeat","return","then","true","until","while",
        "continue","export","type"
    };
    static const QStringList builtins = {
        "assert","error","getmetatable","ipairs","next","pairs","pcall","print",
        "rawequal","rawget","rawlen","rawset","select","setmetatable","tonumber",
        "tostring","type","typeof","unpack","xpcall","math","string","table","task"
    };

    for (const QString& word : keywords)
    {
        QRegularExpression re(QString("\\b%1\\b").arg(QRegularExpression::escape(word)));
        auto it = re.globalMatch(text);
        while (it.hasNext()) {
            auto match = it.next();
            setFormat(match.capturedStart(), match.capturedLength(), keyword_);
        }
    }
    for (const QString& word : builtins)
    {
        QRegularExpression re(QString("\\b%1\\b").arg(QRegularExpression::escape(word)));
        auto it = re.globalMatch(text);
        while (it.hasNext()) {
            auto match = it.next();
            setFormat(match.capturedStart(), match.capturedLength(), builtin_);
        }
    }

    QRegularExpression numberRe(R"(\b\d+(\.\d+)?\b)");
    auto numbers = numberRe.globalMatch(text);
    while (numbers.hasNext()) {
        auto m = numbers.next();
        setFormat(m.capturedStart(), m.capturedLength(), number_);
    }

    QRegularExpression stringRe(R"("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')");
    auto strings = stringRe.globalMatch(text);
    while (strings.hasNext()) {
        auto m = strings.next();
        setFormat(m.capturedStart(), m.capturedLength(), string_);
    }

    const int commentStart = text.indexOf("--");
    if (commentStart >= 0)
        setFormat(commentStart, text.length() - commentStart, comment_);
}
