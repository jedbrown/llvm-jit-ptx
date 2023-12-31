target datalayout = "e-i64:64-i128:128-v16:16-v32:32-n16:32:64"
target triple = "nvptx64-nvidia-cuda"

; Intrinsic to read X component of thread ID
declare i32 @llvm.nvvm.read.ptx.sreg.tid.x() readnone nounwind

define void @kernel(float addrspace(1)* %A,
                    float addrspace(1)* %B,
                    float addrspace(1)* %C) {
entry:
  ; What is my ID?
  %id = tail call i32 @llvm.nvvm.read.ptx.sreg.tid.x() readnone nounwind

  ; Compute pointers into A, B, and C
  %ptrA = getelementptr float, float addrspace(1)* %A, i32 %id
  %ptrB = getelementptr float, float addrspace(1)* %B, i32 %id
  %ptrC = getelementptr float, float addrspace(1)* %C, i32 %id

  ; Read A, B
  %valA = load float, float addrspace(1)* %ptrA, align 4
  %valB = load float, float addrspace(1)* %ptrB, align 4

  ; Compute C = A + B
  ;; %valC = fadd float %valA, %valB
  %valC = tail call float @dfunc_d_swirl(float noundef %valA, float noundef %valB)

  ; Store back to C
  store float %valC, float addrspace(1)* %ptrC, align 4

  ret void
}

declare dso_local noundef float @dfunc_d_swirl(float noundef, float noundef)

!nvvm.annotations = !{!0}
!0 = !{void (float addrspace(1)*,
             float addrspace(1)*,
             float addrspace(1)*)* @kernel, !"kernel", i32 1}
