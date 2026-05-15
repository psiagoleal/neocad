cmake_minimum_required(VERSION 3.24)

if(NOT DEFINED NEOCAD_WINDOWS_FIXED_RUNTIME_DIR OR NEOCAD_WINDOWS_FIXED_RUNTIME_DIR STREQUAL "")
	message(FATAL_ERROR
		"NEOCAD_WINDOWS_FIXED_RUNTIME_DIR is not set. Extract the WebView2 Fixed Runtime to .webview2/fixed-runtime-x64 or configure another directory before building the fixed-runtime targets."
	)
endif()

if(NOT IS_DIRECTORY "${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR}")
	message(FATAL_ERROR
		"NEOCAD_WINDOWS_FIXED_RUNTIME_DIR does not exist: ${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR}"
	)
endif()

if(NOT EXISTS "${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR}/msedgewebview2.exe")
	message(FATAL_ERROR
		"Expected msedgewebview2.exe inside ${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR}. Extract the .cab contents directly into that directory."
	)
endif()

message(STATUS "Using WebView2 Fixed Runtime from: ${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR}")
