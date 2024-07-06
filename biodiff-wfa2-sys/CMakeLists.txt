cmake_minimum_required(VERSION 3.16)
project(wfa2lib)

set(CMAKE_CXX_STANDARD 17)
set(WFA2_PATH WFA2-lib)

include(FeatureSummary)

#find_package(PkgConfig REQUIRED)

feature_summary(
  FATAL_ON_MISSING_REQUIRED_PACKAGES
  WHAT REQUIRED_PACKAGES_NOT_FOUND)

include(CheckIPOSupported)
check_ipo_supported(RESULT LTO_SUPPORTED OUTPUT LTO_CHECK_ERROR)
if (LTO_SUPPORTED)
  set(CMAKE_C_COMPILE_OPTIONS_IPO "-flto=full")
endif()

include(FindOpenMP)
if (NOT OpenMP_FOUND)
  message(WARNING "OpenMP not found!")
endif()

# ---- Dependencies

find_package(Threads)
set_package_properties(Threads PROPERTIES TYPE REQUIRED)

# ---- Build switches
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# ---- Include files
file(GLOB INCLUDES
  ${WFA2_PATH}/wavefront/*.h*
  )
file(GLOB UTILS_INCLUDES
  ${WFA2_PATH}/utils/*.h*
  )
file(GLOB ALIGNMENT_INCLUDES
  ${WFA2_PATH}/alignment/*.h*
  )
file(GLOB SYSTEM_INCLUDES
  ${WFA2_PATH}/system/*.h*
  )

set(wfa2lib_SOURCE
  ${WFA2_PATH}/wavefront/wavefront_align.c
  ${WFA2_PATH}/wavefront/wavefront_aligner.c
  ${WFA2_PATH}/wavefront/wavefront_attributes.c
  ${WFA2_PATH}/wavefront/wavefront_backtrace_buffer.c
  ${WFA2_PATH}/wavefront/wavefront_backtrace.c
  ${WFA2_PATH}/wavefront/wavefront_backtrace_offload.c
  ${WFA2_PATH}/wavefront/wavefront_bialign.c
  ${WFA2_PATH}/wavefront/wavefront_bialigner.c
  ${WFA2_PATH}/wavefront/wavefront.c
  ${WFA2_PATH}/wavefront/wavefront_components.c
  ${WFA2_PATH}/wavefront/wavefront_compute_affine2p.c
  ${WFA2_PATH}/wavefront/wavefront_compute_affine.c
  ${WFA2_PATH}/wavefront/wavefront_compute.c
  ${WFA2_PATH}/wavefront/wavefront_compute_edit.c
  ${WFA2_PATH}/wavefront/wavefront_compute_linear.c
  ${WFA2_PATH}/wavefront/wavefront_debug.c
  ${WFA2_PATH}/wavefront/wavefront_display.c
  ${WFA2_PATH}/wavefront/wavefront_extend.c
  ${WFA2_PATH}/wavefront/wavefront_heuristic.c
  ${WFA2_PATH}/wavefront/wavefront_pcigar.c
  ${WFA2_PATH}/wavefront/wavefront_penalties.c
  ${WFA2_PATH}/wavefront/wavefront_plot.c
  ${WFA2_PATH}/wavefront/wavefront_sequences.c
  ${WFA2_PATH}/wavefront/wavefront_slab.c
  ${WFA2_PATH}/wavefront/wavefront_unialign.c
  ${WFA2_PATH}/wavefront/wavefront_termination.c
  ${WFA2_PATH}/wavefront/wavefront_extend_kernels_avx.c
  ${WFA2_PATH}/wavefront/wavefront_extend_kernels.c
  ${WFA2_PATH}/system/mm_stack.c
  ${WFA2_PATH}/system/mm_allocator.c
  ${WFA2_PATH}/system/profiler_counter.c
  ${WFA2_PATH}/system/profiler_timer.c
  ${WFA2_PATH}/utils/bitmap.c
  ${WFA2_PATH}/utils/dna_text.c
  ${WFA2_PATH}/utils/sequence_buffer.c
  ${WFA2_PATH}/utils/vector.c
  ${WFA2_PATH}/utils/commons.c
  ${WFA2_PATH}/utils/heatmap.c
  ${WFA2_PATH}/alignment/affine2p_penalties.c
  ${WFA2_PATH}/alignment/affine_penalties.c
  ${WFA2_PATH}/alignment/cigar.c
  ${WFA2_PATH}/alignment/score_matrix.c
)

add_library(wfa2_static ${wfa2lib_SOURCE})
add_library(wfa2 SHARED ${wfa2lib_SOURCE})
set_target_properties(wfa2_static PROPERTIES OUTPUT_NAME wfa2)
set_target_properties(wfa2 PROPERTIES SOVERSION 0)
target_include_directories(wfa2 PUBLIC ${WFA2_PATH} ${WFA2_PATH}/wavefront ${WFA2_PATH}/utils)
target_include_directories(wfa2_static PUBLIC ${WFA2_PATH} ${WFA2_PATH}/wavefront ${WFA2_PATH}/utils)
if (LTO_SUPPORTED)
  set_property(TARGET wfa2_static wfa2 PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
else()
  message(WARNING "LTO is not supported: ${LTO_CHECK_ERROR}")
endif()
if (OpenMP_FOUND)
  target_compile_definitions(wfa2_static PRIVATE "WFA_PARALLEL")
  target_compile_definitions(wfa2 PRIVATE "WFA_PARALLEL")
  target_compile_options(wfa2_static PRIVATE ${OpenMP_C_FLAGS} -Xclang --linker-option=-l${OpenMP_C_LIB_NAMES})
  target_compile_options(wfa2 PRIVATE ${OpenMP_C_FLAGS})
  target_link_libraries(wfa2 ${OpenMP_C_LIB_NAMES})
endif()
add_library(wfa2::wfa2 ALIAS wfa2)
add_library(wfa2::wfa2_static ALIAS wfa2_static)

# ---- Get version

file (STRINGS ${WFA2_PATH}/VERSION.txt BUILD_NUMBER)
add_compile_definitions(WFA2LIB_VERSION="${BUILD_NUMBER}")
add_compile_definitions(VERSION="${BUILD_NUMBER}")

set(wfa2lib_LIBS)

# add_dependencies(wfa2lib ${wfa2lib_DEPS})

# ---- Build all

# ---- Install

install(TARGETS wfa2_static ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR} )
