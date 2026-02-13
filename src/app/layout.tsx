/* eslint-disable @typescript-eslint/no-unused-vars */
import type React from "react"
import type { Metadata } from "next"
import { Geist, Geist_Mono } from "next/font/google"
import "./globals.css"
import { Toaster } from "@/src/components/ui/sonner"

const _geist = Geist({ subsets: ["latin"] })
const _geistMono = Geist_Mono({ subsets: ["latin"] })

export const metadata: Metadata = {
  title: "SEO Insikt crawler",
  description: "Analyze websites for SEO issues and get actionable recommendations",
}

import { SettingsDialog } from "@/src/components/settings/settings-dialog"
import { UIProvider } from "@/src/context/UIContext"

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en" className="dark">
      <head>
        {process.env.NODE_ENV === "development" && (
          <script
            crossOrigin="anonymous"
            src="https://unpkg.com/react-scan/dist/auto.global.js"
          />
        )}
      </head>
      <body className={`font-sans antialiased`}>
        <UIProvider>
          {children}
          <Toaster />
          <SettingsDialog />
        </UIProvider>
      </body>
    </html>
  )
}
