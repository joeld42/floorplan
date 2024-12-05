cmake_minimum_required(VERSION 3.24...3.30)

project(proto1 VERSION 0.0.1
                  DESCRIPTION "Proto1"
                  LANGUAGES CXX)

include(FetchContent)

# Generate compile_commands.json
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Dependencies
set(RAYLIB_VERSION 5.0)

FetchContent_Declare(
    raylib
    DOWNLOAD_EXTRACT_TIMESTAMP OFF
    URL https://github.com/raysan5/raylib/archive/refs/tags/${RAYLIB_VERSION}.tar.gz
    FIND_PACKAGE_ARGS
)

FetchContent_MakeAvailable(raylib)


#set ( RAYLIB_HOME /Users/joeld/Toolkits/raylib/raylib )
set ( RAYGUI_HOME c:/Toolkits/raygui )
#set ( RAYGUI_HOME /Users/joeld/Toolkits/raylib/raygui )

include_directories(
#	${RAYLIB_HOME}/src
	${RAYGUI_HOME}/src
#	/usr/local/include	
)

#link_directories(
#	${RAYLIB_HOME}/build/src/
#	/usr/local/Cellar/glfw3/3.2.1/lib 	
#)

if (APPLE)
set ( LIBS ${LIBS}
		"-framework IOKit"
		"-framework OpenGL"
		"-framework OpenAl"
		"-framework Cocoa"
		"-framework AudioToolbox"		
      )
endif()

#set(raylib_VERBOSE 1)


#set ( CMAKE_CXX_FLAGS -Wno-nullability-completeness )
set ( CMAKE_CXX_STANDARD 14)

# =========================================================
# Low-Level Game GUI
# =========================================================
add_executable (proto1

		proto1.cpp
)

target_compile_options( proto1 PUBLIC -fno-rtti)
target_link_libraries( proto1 raylib)
# target_include_directories( proto1 PRIVATE tk_util )
set_target_properties( proto1 PROPERTIES
	 					RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/ )
