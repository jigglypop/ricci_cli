# Ricci CLI Assets

이 폴더에는 Ricci CLI의 아이콘 및 리소스 파일들이 들어갑니다.

## 아이콘 파일
- `ricci.ico` - Windows 아이콘 (16x16, 32x32, 48x48, 256x256)
- `ricci.png` - PNG 버전 (512x512)

## 아이콘 제작 방법
1. 온라인 변환기 사용: https://www.icoconverter.com/
2. 또는 PowerShell에서:
   ```powershell
   # PNG를 ICO로 변환 (ImageMagick 필요)
   magick convert ricci.png -define icon:auto-resize=256,48,32,16 ricci.ico
   ``` 