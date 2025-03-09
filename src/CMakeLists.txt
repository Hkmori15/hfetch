cmake_minimum_required(VERSION 3.15)
project(hfetch VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_BUILD_TYPE Release)
set(CMAKE_CXX_FLAGS_RELEASE "-O3")

add_executable(hfetch main.cpp)
target_link_libraries(hfetch PRIVATE stdc++fs)

install(TARGETS hfetch DESTINATION bin)