cmake_minimum_required(VERSION 3.15)
project(Sudoku VERSION 0.1.0)

include(CTest)
enable_testing()

add_library(Sudoku libs/protobuf/generated/*cc cpp/server/src/*cpp)

set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
