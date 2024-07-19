#Requires AutoHotkey v2.0
;; 自动执行段
;; 脚本设置
#SingleInstance force
SendMode "Input" ; 设置模拟方式
SetKeyDelay 30, 25 ; SendPlay模式延迟
SetWorkingDir A_ScriptDir ; 设置工作目录
SetTitleMatchMode 3
SetCapsLockState 0
SetWinDelay 20
CoordMode "Mouse"

#Include basic.ahk
#Include yuanshen_c.ahk
#Include gui.ahk
#Include yuanshen_h.ahk


; #HotIf WinActive("原神")
; ~F2:: {
;   A_Clipboard := httpRequest("http://47.94.95.163:8887/r/123/poll?user=🦈")
;   _auto_enter_uniq()
; }


OnClipboardChange WatchClipboard
WatchClipboard(data) {
  if data != 1 {
    return
  }
  text := A_Clipboard

  ; if WinActive("QQ频道") or WinActive("自助餐") or WinActive("WeYS") or WinActive("ahk_exe Cursor.exe") or WinActive("ahk_exe Code.exe") {
  ;   _auto_enter_uniq()
  ; }
  if WinActive("WeYS") and RegExMatch(text, "----") {
    _login()
  }
}

#HotIf

; F12:: ExitApp


; Alt+中键移动窗口
!MButton::
{
  MouseGetPos &oriX, &oriY, &hwnd
  WinGetPos &winX, &winY, , , hwnd
  Loop
  {
    if !GetKeyState("MButton", "P")
      break
    MouseGetPos &x, &y
    offsetX := x - oriX
    offsetY := y - oriY
    toX := (winX + offsetX)
    toY := (winY + offsetY)
    WinMove toX, toY, , , hwnd
    ToolTip Format("({1}, {2})", toX, toY)
  }
  ToolTip
}
; Alt+右键缩放窗口
!RButton::
{
  MouseGetPos &oriX, &oriY, &hwnd
  WinGetPos , , &winX, &winY, hwnd
  Loop
  {
    if !GetKeyState("RButton", "P")
      break
    MouseGetPos &x, &y
    offsetX := x - oriX
    offsetY := y - oriY
    toX := (winX + offsetX)
    toY := (winY + offsetY)
    WinMove , , toX, toY, hwnd
    ToolTip Format("({1}, {2})", toX, toY)
  }
  ToolTip
}

#HotIf
; 静音当前程序
*Launch_Media:: {
  MouseGetPos(, , &hwnd)
  ; hwnd := GetForegroundWindow()
  p := WinGetProcessName(hwnd)
  msg("静音当前程序：" . p . " (" . GetKeyState("Shift", "P") . ")")
  setProgramVol(p, GetKeyState("Shift", "P") ? 1 : 0)
}

#HotIf WinActive("ahk_exe ZenlessZoneZero.exe")
r:: {
  f() {
    Click
    Send "eq{Shift}{Space}"
  }
  triggerLoop("绝区零战斗", f, 33)
}