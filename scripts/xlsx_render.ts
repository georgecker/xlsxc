function main(workbook: ExcelScript.Workbook) {
  // Expecting only supporting RGB not RGBa => 3 bytes represent on pixel
  let sheet = workbook.getWorksheet("Sheet 1");
  initFormat(sheet, 1, 10);

  readRender(sheet);
}

function readRender(sheet: ExcelScript.Worksheet) {
  let frameIndex = 1;
  while (true) {
    let currentCell = sheet.getCell(frameIndex, 0);
    if (currentCell == null) {
      break;
    }

    let frameData = currentCell.getText();
    if (frameData == "") {
      break;
    }

    let split = frameData.split(':');
    let width = parseInt(split[0]);
    let pixelData = split[1];
    let chunk_size = 2 * 3;
    for (let i = 0; i < pixelData.length; i += chunk_size) {
      let rgb = pixelData.slice(i, i + chunk_size);
      let pixelIndex = (i / chunk_size);
      let row = Math.floor(pixelIndex / width);
      let column = (pixelIndex % width) + 1;

      setPixel(sheet, row, column, rgb);
    }

    frameIndex += 1;
  }
}

function initFormat(sheet: ExcelScript.Worksheet, width_based_char: number, height_in_pixel: number) {
  sheet.setStandardWidth(width_based_char);
  let currentCell = sheet.getCell(1, 0);
  let frameData = currentCell.getText();

  let split = frameData.split(':');
  let width = parseInt(split[0]);
  let pixelData = split[1];
  let row_count = (pixelData.length - 1) / width;

  for (let row = 1; row <= row_count; row += 1) {
    let cell = sheet.getCell(row, 1);
    let cellFormat = cell.getFormat();
    cellFormat.setRowHeight(height_in_pixel);
  }
}

function setPixel(sheet: ExcelScript.Worksheet, row: number, column: number, rgb: string) {
  let cell = sheet.getCell(row, column);
  let cellFormat = cell.getFormat();
  cell.setValue(" ");
  cellFormat.getFill().setColor(rgb);
}
