import camelot

tables = camelot.read_pdf("byelections_2025.pdf", pages="all")

tables.export("byelections_2025", f="json", compress=True)

print(tables)
