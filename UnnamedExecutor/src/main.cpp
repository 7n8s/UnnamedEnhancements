#include <QApplication>
#include <QFont>
#include "MainWindow.h"

int main(int argc, char* argv[])
{
    QApplication app(argc, argv);
    QApplication::setApplicationName("Unnamed Executor");
    QApplication::setOrganizationName("Unnamed");
    QApplication::setApplicationVersion("0.2.0");
    MainWindow window;
    window.show();
    return app.exec();
}
